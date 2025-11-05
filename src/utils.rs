use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
};

use crate::AppError;

pub fn validate_buff(input_path: &str) -> Result<(bool, File), AppError> {
    // according to Docs:
    // pub fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
    //    OpenOptions::new().read(true).open(path.as_ref())
    // }
    // Attempts to open a file in read-only mode with buffering.
    //
    // See the [`OpenOptions::open`] method, the [`BufReader`][io::BufReader] type,
    // and the [`BufRead`][io::BufRead] trait for more details
    // So, this is buffered already.
    // Check for commented out buffer_capacity at main to tweak buffer size memory in order to
    // avoid bloating memory consumption
    let mut file =
        File::open(input_path).map_err(|_| AppError::FileNotFound(input_path.to_string()))?;
    let mut reader = BufReader::new(file.try_clone()?);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    // Trim and check whether it matches our expected header
    let header_line = first_line.trim().replace(' ', "");
    let has_headers = header_line.eq_ignore_ascii_case("type,client,tx,amount");
    // reset cursor in order to avoid reloading file
    file.seek(SeekFrom::Start(0))?;
    Ok((has_headers, file))
}

pub fn trunc_decimals(value: f32, decimals: u32) -> f32 {
    let factor = 10f32.powi(decimals as i32);
    (value * factor).trunc() / factor
}
