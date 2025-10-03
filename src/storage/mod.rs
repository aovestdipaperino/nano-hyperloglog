mod file;

#[cfg(feature = "elasticsearch-storage")]
mod elasticsearch;

pub use file::FileStorage;

#[cfg(feature = "elasticsearch-storage")]
pub use elasticsearch::ElasticsearchStorage;

use crate::{HyperLogLog, Result};
use async_trait::async_trait;

/// Storage backend for HyperLogLog structures
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store a HyperLogLog with given key
    async fn store(&self, key: &str, hll: &HyperLogLog) -> Result<()>;

    /// Load a HyperLogLog by key
    async fn load(&self, key: &str) -> Result<HyperLogLog>;

    /// Delete a HyperLogLog by key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// List all keys (for debugging/admin purposes)
    async fn list_keys(&self) -> Result<Vec<String>>;
}
