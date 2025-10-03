//! # nano-hyperloglog
//!
//! A high-performance HyperLogLog implementation for cardinality estimation with pluggable storage backends.
//!
//! ## What is HyperLogLog?
//!
//! HyperLogLog is a probabilistic data structure for estimating the cardinality (number of unique elements)
//! of a dataset. It offers remarkable space efficiency: you can count billions of unique items using just
//! a few kilobytes of memory, with typical accuracy within 2% of the true count.
//!
//! ## Features
//!
//! - **Fixed memory usage**: Count billions of items with ~16KB (configurable via precision)
//! - **High accuracy**: Typically within 0.8-2% of true count (depending on precision)
//! - **Mergeable**: Combine HyperLogLogs from multiple sources with simple union operations
//! - **Pluggable storage**: File-based or Elasticsearch backends for persistence
//! - **Redis-compatible API**: Optional HTTP server with PFADD/PFCOUNT/PFMERGE endpoints
//! - **Type-safe**: Leverage Rust's type system for compile-time guarantees
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! nano-hyperloglog = "0.1"
//! ```
//!
//! Basic usage:
//!
//! ```rust
//! use hyperloglog::HyperLogLog;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a HyperLogLog with precision 14 (16KB memory, ~0.8% error)
//! let mut hll = HyperLogLog::new(14)?;
//!
//! // Add elements
//! for i in 0..10000 {
//!     hll.add(&i);
//! }
//!
//! // Get estimated count
//! let count = hll.count();
//! println!("Estimated unique count: {}", count); // ~10000
//! # Ok(())
//! # }
//! ```
//!
//! ## Merging HyperLogLogs
//!
//! HyperLogLogs can be merged to combine counts from multiple sources:
//!
//! ```rust
//! use hyperloglog::HyperLogLog;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut server1 = HyperLogLog::new(14)?;
//! let mut server2 = HyperLogLog::new(14)?;
//!
//! // Each server tracks different users
//! for i in 0..5000 {
//!     server1.add(&i);
//! }
//! for i in 5000..10000 {
//!     server2.add(&i);
//! }
//!
//! // Merge to get total unique count
//! server1.merge(&server2)?;
//! let total = server1.count(); // ~10000
//! # Ok(())
//! # }
//! ```
//!
//! ## Persistent Storage
//!
//! Use storage backends to persist HyperLogLogs:
//!
//! ```rust,no_run
//! use hyperloglog::{HyperLogLog, Storage, storage::FileStorage};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = FileStorage::new("./data").await?;
//!
//! let mut hll = HyperLogLog::new(14)?;
//! hll.add_str("user123");
//! hll.add_str("user456");
//!
//! // Store
//! storage.store("daily_visitors", &hll).await?;
//!
//! // Load later
//! let loaded = storage.load("daily_visitors").await?;
//! println!("Count: {}", loaded.count());
//! # Ok(())
//! # }
//! ```
//!
//! ## Precision and Memory Tradeoffs
//!
//! | Precision | Memory  | Standard Error |
//! |-----------|---------|----------------|
//! | 10        | 1 KB    | ±1.625%        |
//! | 12        | 4 KB    | ±0.813%        |
//! | 14        | 16 KB   | ±0.406%        |
//! | 16        | 64 KB   | ±0.203%        |
//!
//! ## Feature Flags
//!
//! - `file-storage` (default): Enable file-based storage backend
//! - `elasticsearch-storage`: Enable Elasticsearch storage backend
//! - `server`: Enable HTTP server with Redis-compatible API
//! - `full`: Enable all features
//!
//! ## Examples
//!
//! See the `examples/` directory for more usage patterns:
//! - `basic_usage.rs` - Simple cardinality estimation
//! - `merging.rs` - Combining HyperLogLogs from multiple sources
//! - `file_storage.rs` - Using persistent file storage
//! - `precision_comparison.rs` - Comparing different precision values
//! - `server.rs` - Running the HTTP server

pub mod hll;
pub mod error;

#[cfg(feature = "file-storage")]
pub mod storage;

#[cfg(feature = "server")]
pub mod api;

pub use hll::HyperLogLog;
pub use error::{HllError, Result};

#[cfg(feature = "file-storage")]
pub use storage::Storage;

#[cfg(feature = "file-storage")]
pub use storage::FileStorage;

#[cfg(feature = "elasticsearch-storage")]
pub use storage::ElasticsearchStorage;
