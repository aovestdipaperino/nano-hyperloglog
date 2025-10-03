//! Example using file-based storage backend.

use hyperloglog::{HyperLogLog, storage::FileStorage, Storage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file storage in a temporary directory
    let storage_path = std::env::temp_dir().join("hll_example");
    let storage = FileStorage::new(&storage_path).await?;

    println!("Using file storage at: {}", storage_path.display());

    // Create and populate a HyperLogLog
    let mut hll = HyperLogLog::new(14)?;
    for i in 0..1000 {
        hll.add_str(&format!("user_{}", i));
    }

    let count_before = hll.count();
    println!("Created HLL with estimated count: {}", count_before);

    // Store it
    storage.store("daily_visitors", &hll).await?;
    println!("Stored HLL to disk");

    // Check if it exists
    let exists = storage.exists("daily_visitors").await?;
    println!("Key exists: {}", exists);

    // Load it back
    let loaded_hll = storage.load("daily_visitors").await?;
    let count_after = loaded_hll.count();

    println!("Loaded HLL with estimated count: {}", count_after);
    println!("Counts match: {}", count_before == count_after);

    // List all keys
    let keys = storage.list_keys().await?;
    println!("All keys in storage: {:?}", keys);

    // Clean up
    storage.delete("daily_visitors").await?;
    println!("Cleaned up storage");

    Ok(())
}
