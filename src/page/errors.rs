use core::fmt::{self, Formatter, Result};
use std::error::Error;

#[derive(Debug)]
pub enum DBErrors {
    InvalidFileHeader(String),
    InvalidPageHeader(String),
}

impl Error for DBErrors {}

impl fmt::Display for DBErrors {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::InvalidFileHeader(msg) => write!(f, "Invalid File Header: {}", msg),
            Self::InvalidPageHeader(msg) => write!(f, "Invalid Page Header: {}", msg),
        }
    }
}
