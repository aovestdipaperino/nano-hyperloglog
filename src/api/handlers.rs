use crate::{HyperLogLog, HllError};
use super::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// Redis PFADD command - Add elements to HyperLogLog
#[derive(Debug, Deserialize)]
pub struct PfAddRequest {
    pub elements: Vec<String>,
}

/// Redis PFCOUNT command - Get cardinality estimate
#[derive(Debug, Serialize)]
pub struct PfCountResponse {
    pub count: u64,
}

/// Redis PFMERGE command - Merge multiple HyperLogLogs
#[derive(Debug, Deserialize)]
pub struct PfMergeRequest {
    pub source_keys: Vec<String>,
}

/// Generic success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for HllError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            HllError::NotFound(key) => (StatusCode::NOT_FOUND, format!("Key not found: {}", key)),
            HllError::InvalidKey(key) => (StatusCode::BAD_REQUEST, format!("Invalid key: {}", key)),
            HllError::InvalidPrecision(p) => {
                (StatusCode::BAD_REQUEST, format!("Invalid precision: {}", p))
            }
            HllError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            HllError::Serialization(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {}", e))
            }
            HllError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("IO error: {}", e)),
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

/// PFADD - Add elements to a HyperLogLog
pub async fn pfadd(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<PfAddRequest>,
) -> Result<Json<SuccessResponse>, HllError> {
    let mut hll = match state.storage().load(&key).await {
        Ok(hll) => hll,
        Err(HllError::NotFound(_)) => HyperLogLog::new(14)?,
        Err(e) => return Err(e),
    };

    for element in &payload.elements {
        hll.add_str(element);
    }

    state.storage().store(&key, &hll).await?;

    Ok(Json(SuccessResponse {
        success: true,
        message: format!("Added {} elements", payload.elements.len()),
    }))
}

/// PFCOUNT - Get cardinality estimate from one or more HyperLogLogs
pub async fn pfcount(
    State(state): State<AppState>,
    Path(keys): Path<String>,
) -> Result<Json<PfCountResponse>, HllError> {
    let key_list: Vec<&str> = keys.split(',').collect();

    if key_list.is_empty() {
        return Ok(Json(PfCountResponse { count: 0 }));
    }

    let mut merged = state.storage().load(key_list[0]).await?;

    for key in &key_list[1..] {
        let hll = state.storage().load(key).await?;
        merged.merge(&hll)?;
    }

    let count = merged.count();

    Ok(Json(PfCountResponse { count }))
}

/// PFMERGE - Merge multiple HyperLogLogs into destination key
pub async fn pfmerge(
    State(state): State<AppState>,
    Path(dest_key): Path<String>,
    Json(payload): Json<PfMergeRequest>,
) -> Result<Json<SuccessResponse>, HllError> {
    if payload.source_keys.is_empty() {
        return Err(HllError::InvalidKey("No source keys provided".to_string()));
    }

    let mut merged = state.storage().load(&payload.source_keys[0]).await?;

    for key in &payload.source_keys[1..] {
        let hll = state.storage().load(key).await?;
        merged.merge(&hll)?;
    }

    state.storage().store(&dest_key, &merged).await?;

    Ok(Json(SuccessResponse {
        success: true,
        message: format!("Merged {} keys into {}", payload.source_keys.len(), dest_key),
    }))
}

/// DELETE - Delete a HyperLogLog key
pub async fn delete(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<SuccessResponse>, HllError> {
    state.storage().delete(&key).await?;

    Ok(Json(SuccessResponse {
        success: true,
        message: format!("Deleted key: {}", key),
    }))
}

/// EXISTS - Check if a key exists
pub async fn exists(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<bool>, HllError> {
    let exists = state.storage().exists(&key).await?;
    Ok(Json(exists))
}

/// LIST - List all keys
pub async fn list_keys(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, HllError> {
    let keys = state.storage().list_keys().await?;
    Ok(Json(keys))
}
