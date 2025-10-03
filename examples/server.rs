//! HTTP server example with Redis-compatible API.
//!
//! Run with: `cargo run --example server --features server`

use hyperloglog::api::{create_router, AppState};
use hyperloglog::storage::FileStorage;
use hyperloglog::Storage;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "elasticsearch-storage")]
use hyperloglog::storage::ElasticsearchStorage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hyperloglog=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Determine storage backend from environment
    let storage_backend = std::env::var("STORAGE_BACKEND")
        .unwrap_or_else(|_| "file".to_string());

    let storage: Arc<dyn Storage> = match storage_backend.as_str() {
        #[cfg(feature = "elasticsearch-storage")]
        "elasticsearch" => {
            let es_url = std::env::var("ELASTICSEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string());
            let index_name = std::env::var("ELASTICSEARCH_INDEX")
                .unwrap_or_else(|_| "hyperloglog".to_string());

            tracing::info!("Using Elasticsearch storage at {} with index {}", es_url, index_name);
            Arc::new(ElasticsearchStorage::with_url(&es_url, index_name)?)
        }
        #[cfg(not(feature = "elasticsearch-storage"))]
        "elasticsearch" => {
            panic!("Elasticsearch storage requested but feature not enabled. Rebuild with --features elasticsearch-storage");
        }
        _ => {
            let base_path = std::env::var("FILE_STORAGE_PATH")
                .unwrap_or_else(|_| "./data".to_string());

            tracing::info!("Using file storage at {}", base_path);
            Arc::new(FileStorage::new(&base_path).await?)
        }
    };

    // Create application state
    let state = AppState::new(storage);

    // Build router
    let app = create_router(state);

    // Get bind address from environment
    let addr = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    tracing::info!("Starting HyperLogLog server on {}", addr);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
