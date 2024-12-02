use std::error::Error;
use std::fmt::{self, Formatter, Result};

#[derive(Debug, Clone)]
pub enum DBError {
    InvalidFileHeader(String),
    InvalidPageHeader(String),
    InvalidVarintSize,
}

impl Error for DBError {}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::InvalidFileHeader(msg) => write!(f, "Invalid File Header: {}", msg),
            Self::InvalidPageHeader(msg) => write!(f, "Invalid Page Header: {}", msg),
            Self::InvalidVarintSize => write!(f, "Invalid Varint Size"),
        }
    }
}
