mod handlers;
mod routes;

pub use routes::create_router;

use crate::storage::Storage;
use std::sync::Arc;

/// Shared application state containing storage backend
#[derive(Clone)]
pub struct AppState {
    storage: Arc<dyn Storage>,
}

impl AppState {
    /// Create new application state with given storage backend
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Get reference to storage backend
    pub fn storage(&self) -> &dyn Storage {
        self.storage.as_ref()
    }
}
