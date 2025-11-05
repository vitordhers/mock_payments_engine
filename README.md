# Payments Engine

A Rust implementation of a **mock payments engine** as requested in take-home challenge.
The engine processes transactions from a CSV input stream, updates client accounts according to defined business rules, and outputs final account states as a CSV to **stdout**.

---

## Overview

This project implements a small, memory-safe, and efficient transaction processor that supports:

- **Deposits**
- **Withdrawals**
- **Disputes**
- **Resolves**
- **Chargebacks**

The input is streamed line-by-line to handle large files without loading them fully into memory.
Each transaction mutates an in-memory `HashMap<u16, User>` simulating a simple ledger database.

At the end of processing, client balances are printed in CSV format with four-decimal precision.

---

## Usage

```bash
$ cargo run -- transactions.csv > accounts.csv
```

- `transactions.csv` — Input file (must follow the required schema)
- `accounts.csv` — Output redirected to a file or printed on screen

---

## Input Format

The input CSV **must include headers**:

```csv
type, client, tx, amount
deposit, 1, 1, 1.0
withdrawal, 1, 2, 0.5
dispute, 1, 1,
resolve, 1, 1,
chargeback, 1, 1,
```

- `type`: `"deposit" | "withdrawal" | "dispute" | "resolve" | "chargeback"`
- `client`: unique client ID (`u16`)
- `tx`: unique transaction ID (`u32`)
- `amount`: decimal number (optional for dispute/resolve/chargeback)

---

## Output Format

Printed to `stdout`:

```csv
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
```

Definitions:

| Column      | Description                                   |
| ----------- | --------------------------------------------- |
| `available` | Funds available for trading, withdrawal, etc. |
| `held`      | Funds held in dispute.                        |
| `total`     | Sum of available and held funds.              |
| `locked`    | Account frozen (after chargeback).            |

---

## Transaction Rules

| Type           | Behavior                                                                       |
| -------------- | ------------------------------------------------------------------------------ |
| **Deposit**    | Increases available and total balance.                                         |
| **Withdrawal** | Decreases available and total balance, only if sufficient funds exist.         |
| **Dispute**    | Marks a deposit as disputed — moves funds from available → held.               |
| **Resolve**    | Resolves a dispute — moves funds from held → available.                        |
| **Chargeback** | Finalizes a dispute — removes disputed funds from total and locks the account. |

---

## Assumptions

To keep behavior consistent and deterministic, the following assumptions were made:

1. **Withdrawals cannot be disputed** — only deposits can enter dispute flow.
2. **Transaction IDs (`tx`) are globally unique** — reused IDs are ignored.
3. **Client IDs (`client`) are unique** — new clients are created on first reference.
4. **Disputes / resolves / chargebacks** referencing nonexistent transactions are **ignored**.
5. **Once locked**, an account **cannot process any further transactions**.
6. **Funds are tracked in ticks (`i32`)** internally to avoid floating-point rounding issues.
7. **Precision:** all printed values show **4 decimal places**, matching prompt expectations.
8. **No persistence** — data is kept only in memory during runtime.
9. **Input rows are assumed to be well-formed** — the CSV file cannot contain syntax or format errors.
10. **Dispute, resolve, and chargeback lines must have a trailing comma** after the transaction ID, e.g.:
```csv
dispute, 1, 1,
resolve, 1, 1,
chargeback, 1, 1,
```
---

## Example

Input (`transactions.csv`):

```csv
type, client, tx, amount
deposit, 1, 1, 1.5
deposit, 1, 2, 2.0
withdrawal, 1, 3, 1.0
dispute, 1, 1,
resolve, 1, 1,
deposit, 2, 4, 2.0
withdrawal, 2, 5, 1.0
deposit, 3, 6, 3.0
dispute, 3, 6,
chargeback, 3, 6,
```

Output:

```csv
client,available,held,total,locked
1,2.5000,0.0000,2.5000,false
2,1.0000,0.0000,1.0000,false
3,0.0000,0.0000,0.0000,true
```

---

## Implementation Details

- **Dependencies:**

  - `csv` — for streaming CSV parsing

- **Key Structures:**

  - `User` — represents an account.
  - `Transaction` — stores side (deposit/withdrawal), amount, and dispute status.
  - `TransactionInput` — sequential input data, parsed directly from CSV rows.
  - `TransactionStatus` — tracks `Normal`, `Disputed`, or `Solved`.

- **Error handling:** all domain and I/O errors are encapsulated in a custom `AppError` enum.

---

## Design Considerations

### Correctness

Each balance computation is deterministic and processing correctness is ensured by Rust's type system. Since the motivation behind this task is ensuring idiomatic Rust knowledge, and Rust type system ensures conciseness, writing a test suite would have been overkill.

### Maintainability

The code is modular:

- `error.rs` → domain errors
- `core.rs` → main structs and enums logic
- `utils.rs` → helper functions
- `main.rs` → CLI orchestration

### Safety

Since all operations are *add* and *sub*, storing amounts as units of ticks (*as per defined in ```static.rs```*) allows us to leverage integer-based accounting, in order to prevent floating-point drift hazard.

### Efficiency

The engine processes transactions in a **streaming fashion**, keeping only current client data in memory.
This allows scaling to large input files (millions of lines) without loading the full dataset.

---

You can also verify output manually:

```bash
cargo run -- sample_data.csv
```

---

**Author:** Vitor
**License:** MIT
**Language:** Rust 🦀
