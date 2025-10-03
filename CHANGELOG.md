# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-XX

### Added

- Initial release of `nano-hyperloglog`
- Core HyperLogLog implementation with configurable precision (4-16 bits)
- `add()` and `add_str()` methods for adding elements
- `count()` method for cardinality estimation with bias corrections
- `merge()` method for combining HyperLogLogs
- Pluggable storage architecture via `Storage` trait
- File-based storage backend (`FileStorage`)
- Elasticsearch storage backend (`ElasticsearchStorage`)
- Redis-compatible HTTP server with endpoints:
  - `POST /pfadd/:key` - Add elements (PFADD)
  - `GET /pfcount/:keys` - Get count (PFCOUNT)
  - `POST /pfmerge/:dest_key` - Merge HLLs (PFMERGE)
  - `DELETE /delete/:key` - Delete key
  - `GET /exists/:key` - Check existence
  - `GET /keys` - List all keys
- Comprehensive error handling with `HllError` type
- Automatic HTTP status code mapping for errors
- JSON serialization/deserialization support
- Feature flags for optional components:
  - `file-storage` (default)
  - `elasticsearch-storage`
  - `server`
  - `full`
- Environment-based configuration for server
- Structured logging with `tracing`
- Examples:
  - `basic_usage.rs` - Simple counting
  - `merging.rs` - Distributed merging
  - `file_storage.rs` - Persistent storage
  - `precision_comparison.rs` - Accuracy/memory tradeoffs
  - `server.rs` - HTTP server
- Comprehensive test suite covering:
  - Precision validation
  - Counting accuracy at various scales
  - Deduplication
  - Merging (disjoint, overlapping, same data)
  - Serialization round-trips
  - Edge cases (empty HLL, precision mismatches)
- Full crate documentation with examples
- README with quick start guide and API examples
- CLAUDE.md for AI-assisted development

### Dependencies

- `serde` 1.0 - Serialization framework
- `serde_json` 1.0 - JSON support
- `thiserror` 1.0 - Error handling
- `twox-hash` 1.6 - Fast hashing
- `async-trait` 0.1 - Async trait support
- `tokio` 1.0 (optional) - Async runtime
- `axum` 0.7 (optional) - HTTP server
- `elasticsearch` 8.5 (optional) - Elasticsearch client
- `tracing` 0.1 (optional) - Logging

## [Unreleased]

### Future Considerations

- Sparse representation for small cardinalities
- Additional storage backends (Redis, PostgreSQL, S3)
- Compression for serialized HyperLogLogs
- Batch operations in storage layer
- Metrics and monitoring endpoints
- gRPC API in addition to HTTP
- Support for HyperLogLog++ improvements
- CLI tool for common operations
- Performance optimizations (SIMD, better hashing)
- WebAssembly support

[0.1.0]: https://github.com/aovestdipaperino/nano-hyperloglog/releases/tag/v0.1.0
