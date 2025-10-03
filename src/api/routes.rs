use super::{handlers, AppState};
use axum::{
    routing::{delete, get, post},
    Router,
};

/// Create the application router with all Redis-compatible endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Redis HyperLogLog commands
        .route("/pfadd/:key", post(handlers::pfadd))
        .route("/pfcount/:keys", get(handlers::pfcount))
        .route("/pfmerge/:dest_key", post(handlers::pfmerge))
        // Additional utility endpoints
        .route("/delete/:key", delete(handlers::delete))
        .route("/exists/:key", get(handlers::exists))
        .route("/keys", get(handlers::list_keys))
        .with_state(state)
}
