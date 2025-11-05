use csv::Error as CsvError;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::io::Error as IoError;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug)]
pub enum AppError {
    MissingArgument,
    FileNotFound(String),
    InvalidFormat(String),
    InvalidRecord(String),
    InvalidTxType(String),
    IoError(IoError),
    CsvError(CsvError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
}

impl From<csv::Error> for AppError {
    fn from(value: CsvError) -> Self {
        AppError::CsvError(value)
    }
}

impl From<IoError> for AppError {
    fn from(value: IoError) -> Self {
        AppError::IoError(value)
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::ParseInt(err)
    }
}

impl From<ParseFloatError> for AppError {
    fn from(err: ParseFloatError) -> Self {
        AppError::ParseFloat(err)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            AppError::MissingArgument => write!(
                f,
                "Usage: cargo run -- <input_file>\nError: missing input file argument"
            ),
            AppError::FileNotFound(path) => write!(f, "File not found: {}", path),
            AppError::InvalidFormat(reason) => write!(f, "Invalid file format: {}", reason),
            AppError::InvalidRecord(record) => {
                write!(f, "Invalid record for creating transaction: {}", record)
            }
            AppError::InvalidTxType(invalid) => write!(f, "Invalid transaction type {}", invalid),
            AppError::IoError(err) => write!(f, "I/O error: {}", err),
            AppError::CsvError(err) => write!(f, "CSV error: {}", err),
            AppError::ParseInt(err) => write!(f, "Parse int error {}", err),
            AppError::ParseFloat(err) => write!(f, "Parse float error {}", err),
        }
    }
}
