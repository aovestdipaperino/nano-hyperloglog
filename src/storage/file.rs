use crate::{HyperLogLog, Result, HllError};
use crate::storage::Storage;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// File-based storage backend for HyperLogLog structures
#[derive(Debug, Clone)]
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    /// Create a new FileStorage with the given base directory
    pub async fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path).await?;

        Ok(Self { base_path })
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        self.base_path.join(format!("{}.hll", key))
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn store(&self, key: &str, hll: &HyperLogLog) -> Result<()> {
        let path = self.key_to_path(key);
        let serialized = serde_json::to_vec(hll)?;

        let mut file = fs::File::create(&path).await?;
        file.write_all(&serialized).await?;
        file.flush().await?;

        Ok(())
    }

    async fn load(&self, key: &str) -> Result<HyperLogLog> {
        let path = self.key_to_path(key);

        if !path.exists() {
            return Err(HllError::NotFound(key.to_string()));
        }

        let mut file = fs::File::open(&path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        let hll = serde_json::from_slice(&contents)?;
        Ok(hll)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.key_to_path(key);

        if path.exists() {
            fs::remove_file(&path).await?;
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "hll" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(key) = stem.to_str() {
                            keys.push(key.to_string());
                        }
                    }
                }
            }
        }

        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_storage() {
        let temp_dir = std::env::temp_dir().join("hll_test");
        let storage = FileStorage::new(&temp_dir).await.unwrap();

        let mut hll = HyperLogLog::new(10).unwrap();
        hll.add_str("test1");
        hll.add_str("test2");

        storage.store("test_key", &hll).await.unwrap();
        assert!(storage.exists("test_key").await.unwrap());

        let loaded = storage.load("test_key").await.unwrap();
        assert_eq!(loaded.precision(), hll.precision());

        storage.delete("test_key").await.unwrap();
        assert!(!storage.exists("test_key").await.unwrap());

        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
