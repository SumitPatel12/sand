use std::error::Error;
use std::fmt::{self, Formatter, Result};

#[derive(Debug, Clone)]
pub enum DBError {
    InvalidFileHeader(String),
    InvalidPageHeader(String),
    InvalidVarintSize,
    InvalidPageType(u8),
    InvalidSerialType(u64),
}

impl Error for DBError {}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::InvalidFileHeader(msg) => write!(f, "{}", msg),
            Self::InvalidPageHeader(msg) => write!(f, "{}", msg),
            Self::InvalidVarintSize => write!(f, "Invalid Varint Size"),
            Self::InvalidPageType(page_type) => write!(
                f,
                "Invalid Page Type: {}. Page Type must be either 2, 5, 10 or 15.",
                page_type
            ),
            Self::InvalidSerialType(serial_type) => {
                write!(f, "Invalid Serial Type: {}", serial_type)
            }
        }
    }
}
