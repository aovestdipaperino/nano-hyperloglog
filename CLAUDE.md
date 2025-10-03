# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`nano-hyperloglog` is a Rust crate providing a Redis-compatible HyperLogLog service with pluggable storage backends. HyperLogLog is a probabilistic data structure for cardinality estimation that provides approximate counts with minimal memory usage.

## Commands

### Build and Run
- `cargo build` - Build the project
- `cargo run` - Run the HyperLogLog server (listens on 0.0.0.0:3000 by default)
- `cargo build --release` - Build optimized release version

### Testing
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test
- `cargo test -- --nocapture` - Run tests with output visible

### Code Quality
- `cargo clippy` - Run linter for code improvements
- `cargo fmt` - Format code according to Rust style guidelines
- `cargo check` - Quick compilation check without producing binary

## Architecture

### Core Components

1. **HyperLogLog Implementation** (`src/hll.rs`)
   - Core probabilistic cardinality estimation algorithm
   - Configurable precision (4-16 bits)
   - Support for merging multiple HyperLogLogs
   - Uses xxHash for element hashing

2. **Storage Layer** (`src/storage/`)
   - Abstract `Storage` trait for pluggable backends
   - **FileStorage** - Local filesystem-based persistence
   - **ElasticsearchStorage** - Elasticsearch-based distributed storage
   - All storage operations are async

3. **REST API** (`src/api/`)
   - Redis-compatible HTTP endpoints using Axum framework
   - Commands: PFADD, PFCOUNT, PFMERGE, plus utilities (DELETE, EXISTS, LIST)
   - JSON request/response format

4. **Server** (`src/main.rs`)
   - Configurable via environment variables
   - Supports both file and Elasticsearch storage backends
   - Structured logging with tracing

### Environment Variables

- `STORAGE_BACKEND` - Storage backend: "file" (default) or "elasticsearch"
- `FILE_STORAGE_PATH` - Base directory for file storage (default: "./data")
- `ELASTICSEARCH_URL` - Elasticsearch URL (default: "http://localhost:9200")
- `ELASTICSEARCH_INDEX` - Elasticsearch index name (default: "hyperloglog")
- `BIND_ADDRESS` - Server bind address (default: "0.0.0.0:3000")

### API Endpoints

- `POST /pfadd/:key` - Add elements to HyperLogLog (Redis PFADD)
- `GET /pfcount/:keys` - Get cardinality estimate (Redis PFCOUNT, supports comma-separated keys)
- `POST /pfmerge/:dest_key` - Merge HyperLogLogs (Redis PFMERGE)
- `DELETE /delete/:key` - Delete a key
- `GET /exists/:key` - Check if key exists
- `GET /keys` - List all keys

### Key Design Patterns

- **Pluggable Storage**: Storage trait allows easy addition of new backends
- **Async-first**: All I/O operations are async using tokio
- **Error Handling**: Custom error types with proper HTTP status mapping
- **Type Safety**: Strong typing throughout with minimal `unwrap()` usage
