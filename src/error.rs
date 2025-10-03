use thiserror::Error;

#[derive(Error, Debug)]
pub enum HllError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("HyperLogLog not found: {0}")]
    NotFound(String),

    #[error("Invalid precision: {0}")]
    InvalidPrecision(u8),
}

pub type Result<T> = std::result::Result<T, HllError>;
