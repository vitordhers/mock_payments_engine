use csv::ReaderBuilder;
use std::collections::HashMap;
use std::env;
use std::io::{Write, stdout};

mod error;
pub use error::*;
mod utils;
pub use utils::*;
mod core;
pub use core::*;
mod r#static;
pub use r#static::*;

fn main() -> Result<(), AppError> {
    // Get input file path from CLI args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(AppError::MissingArgument);
    }
    let input_path = &args[1];
    let (has_headers, file) = validate_buff(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(has_headers)
        // .buffer_capacity(64 * 1024) // for further on this, check validate_buff comments
        .from_reader(file);

    let stdout = stdout();
    let mut handle = stdout.lock();

    // according to GPT:
    // records() returns a StringRecordsIter<'a, R> — where R: io::Read.
    // That iterator wraps your reader’s R (in your case, a File), and calls .fill_buf() on it when needed.
    // in short: It pulls bytes incrementally from the file handle using buffered I/O.

    let mut mock_db: HashMap<u16, User> = HashMap::new();

    for (i, result) in reader.records().enumerate() {
        let record =
            result.map_err(|e| AppError::InvalidFormat(format!("Line {}: {}", i + 1, e)))?;
        let tx_input = TransactionInput::try_from_string_record(record)?;
        let client_id = tx_input.client_id();
        let client = mock_db.entry(client_id).or_insert(User::new(client_id));
        client.process_tx_input(tx_input)?;
    }

    writeln!(handle, "{}", User::csv_header())?;
    // since on output, client_id order is irrelevant, we're able to iterate over hashmap's values
    for client in mock_db.values() {
        writeln!(handle, "{}", client.to_csv_row())?;
    }

    Ok(())
}
