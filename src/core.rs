use csv::StringRecord;
use std::collections::HashMap;

use crate::{AppError, TICK_SIZE, trunc_decimals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Normal,
    Disputed,
    Solved(bool), // true if chargeback occurred
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionSide {
    Deposit,
    Withdrawal,
}

impl std::str::FromStr for TransactionType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => Ok(Self::Deposit),
            "withdrawal" => Ok(Self::Withdrawal),
            "dispute" => Ok(Self::Dispute),
            "resolve" => Ok(Self::Resolve),
            "chargeback" => Ok(Self::Chargeback),
            _ => Err(AppError::InvalidTxType(s.to_string())),
        }
    }
}

pub enum TransactionInput {
    Deposit(u32, u16, i32),
    Withdrawal(u32, u16, i32),
    Dispute(u32, u16),
    Resolve(u32, u16),
    Chargeback(u32, u16),
}

impl TransactionInput {
    /// assumes [type, client, tx, amount]
    pub fn try_from_string_record(value: StringRecord) -> Result<Self, AppError> {
        let is_non_numeric_tx = value[3].is_empty();
        // sanitize
        let value: Vec<String> = value.iter().map(|s| s.trim().to_lowercase()).collect();
        let tx_type: TransactionType = value[0]
            .parse()
            .unwrap_or_else(|_| panic!("{} to be parsed as tx_type", value[0]));
        if let (true, TransactionType::Deposit | TransactionType::Withdrawal) =
            (is_non_numeric_tx, tx_type)
        {
            return Err(AppError::InvalidRecord(value.join(",").to_string()));
        }

        let client_id = value[1].parse::<u16>()?;
        let id = value[2].parse::<u32>()?;
        match tx_type {
            TransactionType::Deposit | TransactionType::Withdrawal => {
                let amount = if let Some(val) = value.get(3) {
                    let value = trunc_decimals(val.parse::<f32>()?, 4);
                    if !value.is_finite() {
                        return Err(AppError::InvalidRecord(format!("{} is not finite", value)));
                    }
                    (value / TICK_SIZE).round() as i32
                } else {
                    return Err(AppError::InvalidRecord(
                        "Deposit | Withdrawal transactions must have amount".to_string(),
                    ));
                };
                match tx_type {
                    TransactionType::Deposit => Ok(Self::Deposit(id, client_id, amount)),
                    TransactionType::Withdrawal => Ok(Self::Withdrawal(id, client_id, amount)),
                    _ => unreachable!(),
                }
            }
            TransactionType::Dispute => Ok(Self::Dispute(id, client_id)),
            TransactionType::Resolve => Ok(Self::Resolve(id, client_id)),
            TransactionType::Chargeback => Ok(Self::Chargeback(id, client_id)),
        }
    }

    fn id(&self) -> u32 {
        match self {
            TransactionInput::Deposit(id, _, _) | TransactionInput::Withdrawal(id, _, _) => *id,
            TransactionInput::Dispute(id, _)
            | TransactionInput::Resolve(id, _)
            | TransactionInput::Chargeback(id, _) => *id,
        }
    }

    pub fn client_id(&self) -> u16 {
        match self {
            TransactionInput::Deposit(_, client_id, _)
            | TransactionInput::Withdrawal(_, client_id, _) => *client_id,
            TransactionInput::Dispute(_, client_id)
            | TransactionInput::Resolve(_, client_id)
            | TransactionInput::Chargeback(_, client_id) => *client_id,
        }
    }
}

pub struct Transaction {
    pub id: u32,
    pub client_id: u16,
    pub status: TransactionStatus,
    pub side: TransactionSide,
    /// since we're dealing only with add_sub ops, we can safely store amount as ticks
    pub amount: i32,
}

impl Transaction {
    fn new(id: u32, client_id: u16, side: TransactionSide, amount: i32) -> Self {
        Self {
            id,
            client_id,
            side,
            status: TransactionStatus::Normal,
            amount,
        }
    }
}

pub struct User {
    pub id: u16,
    pub locked: bool,
    pub transactions: HashMap<u32, Transaction>,
}

impl User {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            locked: false,
            transactions: HashMap::new(),
        }
    }

    pub fn csv_header() -> &'static str {
        "client,available,held,total,locked"
    }

    pub fn process_tx_input(&mut self, tx: TransactionInput) -> Result<(), AppError> {
        assert!(
            tx.client_id() == self.id,
            "tx's client_id's must be the same as client.id"
        );
        if self.locked {
            // client is frozen and no longer accepts transactions
            return Ok(());
        }
        let tx_id = tx.id();
        match (tx, self.transactions.get_mut(&tx_id)) {
            (TransactionInput::Deposit(id, client_id, amount), None) => {
                self.transactions.insert(
                    id,
                    Transaction::new(id, client_id, TransactionSide::Deposit, amount),
                );
            }
            (TransactionInput::Withdrawal(id, client_id, amount), None) => {
                // if insufficient funds, ignore
                if self.available() >= amount {
                    self.transactions.insert(
                        id,
                        Transaction::new(id, client_id, TransactionSide::Withdrawal, amount),
                    );
                }
            }
            (TransactionInput::Dispute(_, _), Some(found_tx)) => {
                if found_tx.side == TransactionSide::Deposit
                    && found_tx.status == TransactionStatus::Normal
                {
                    found_tx.status = TransactionStatus::Disputed
                }
            }
            (TransactionInput::Resolve(_, _), Some(found_tx)) => {
                if found_tx.status == TransactionStatus::Disputed {
                    found_tx.status = TransactionStatus::Solved(false)
                }
            }
            (TransactionInput::Chargeback(_, _), Some(found_tx)) => {
                if found_tx.status == TransactionStatus::Disputed {
                    found_tx.status = TransactionStatus::Solved(true);
                    self.locked = true;
                }
            }
            // ignore duplicate id numeric and non-numeric but previously absent inputs
            (_, _) => {}
        }

        Ok(())
    }

    fn available(&self) -> i32 {
        self.transactions
            .values()
            .fold(0, |acc, tx| match (tx.side, tx.status) {
                // normal or resolved deposits increase available
                (TransactionSide::Deposit, TransactionStatus::Normal)
                | (TransactionSide::Deposit, TransactionStatus::Solved(false)) => acc + tx.amount,
                // withdrawals always subtract immediately (disputed withdrawals are ignored)
                (TransactionSide::Withdrawal, TransactionStatus::Normal)
                | (TransactionSide::Withdrawal, TransactionStatus::Solved(false)) => {
                    acc - tx.amount
                }
                // disputed or chargebacked deposits are not available
                _ => acc,
            })
            .max(0) // ensures amount >= 0
    }

    fn held(&self) -> i32 {
        self.transactions
            .values()
            .fold(0, |acc, tx| match (tx.side, tx.status) {
                // deposits under dispute are held
                (TransactionSide::Deposit, TransactionStatus::Disputed) => acc + tx.amount,
                _ => acc,
            })
    }

    fn total(&self) -> i32 {
        self.available() + self.held()
    }

    pub fn to_csv_row(&self) -> String {
        let available = self.available() as f32 * TICK_SIZE;
        let held = self.held() as f32 * TICK_SIZE;
        let total = self.total() as f32 * TICK_SIZE;

        format!(
            "{},{:.4},{:.4},{:.4},{}",
            self.id, available, held, total, self.locked
        )
    }
}
