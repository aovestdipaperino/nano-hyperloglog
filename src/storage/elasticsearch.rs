use crate::{HyperLogLog, Result, HllError};
use crate::storage::Storage;
use async_trait::async_trait;
use elasticsearch::{
    Elasticsearch, http::transport::Transport, DeleteParts, GetParts, IndexParts, SearchParts,
};
use serde_json::{json, Value};

/// Elasticsearch-based storage backend for HyperLogLog structures
#[derive(Clone)]
pub struct ElasticsearchStorage {
    client: Elasticsearch,
    index_name: String,
}

impl ElasticsearchStorage {
    /// Create a new ElasticsearchStorage with default connection
    pub fn new(index_name: impl Into<String>) -> Result<Self> {
        let transport = Transport::single_node("http://localhost:9200")
            .map_err(|e| HllError::Storage(format!("Failed to create transport: {}", e)))?;

        let client = Elasticsearch::new(transport);

        Ok(Self {
            client,
            index_name: index_name.into(),
        })
    }

    /// Create a new ElasticsearchStorage with custom URL
    pub fn with_url(url: &str, index_name: impl Into<String>) -> Result<Self> {
        let transport = Transport::single_node(url)
            .map_err(|e| HllError::Storage(format!("Failed to create transport: {}", e)))?;

        let client = Elasticsearch::new(transport);

        Ok(Self {
            client,
            index_name: index_name.into(),
        })
    }
}

#[async_trait]
impl Storage for ElasticsearchStorage {
    async fn store(&self, key: &str, hll: &HyperLogLog) -> Result<()> {
        let serialized = serde_json::to_string(hll)?;

        let response = self
            .client
            .index(IndexParts::IndexId(&self.index_name, key))
            .body(json!({
                "key": key,
                "hll_data": serialized,
                "precision": hll.precision(),
            }))
            .send()
            .await
            .map_err(|e| HllError::Storage(format!("Failed to store: {}", e)))?;

        if !response.status_code().is_success() {
            return Err(HllError::Storage(format!(
                "Elasticsearch returned status: {}",
                response.status_code()
            )));
        }

        Ok(())
    }

    async fn load(&self, key: &str) -> Result<HyperLogLog> {
        let response = self
            .client
            .get(GetParts::IndexId(&self.index_name, key))
            .send()
            .await
            .map_err(|e| HllError::Storage(format!("Failed to load: {}", e)))?;

        if !response.status_code().is_success() {
            if response.status_code() == 404 {
                return Err(HllError::NotFound(key.to_string()));
            }
            return Err(HllError::Storage(format!(
                "Elasticsearch returned status: {}",
                response.status_code()
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| HllError::Storage(format!("Failed to parse response: {}", e)))?;

        let hll_data = body["_source"]["hll_data"]
            .as_str()
            .ok_or_else(|| HllError::Storage("Missing hll_data field".to_string()))?;

        let hll: HyperLogLog = serde_json::from_str(hll_data)?;
        Ok(hll)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let response = self
            .client
            .delete(DeleteParts::IndexId(&self.index_name, key))
            .send()
            .await
            .map_err(|e| HllError::Storage(format!("Failed to delete: {}", e)))?;

        if !response.status_code().is_success() && response.status_code() != 404 {
            return Err(HllError::Storage(format!(
                "Elasticsearch returned status: {}",
                response.status_code()
            )));
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let response = self
            .client
            .get(GetParts::IndexId(&self.index_name, key))
            .send()
            .await
            .map_err(|e| HllError::Storage(format!("Failed to check existence: {}", e)))?;

        Ok(response.status_code().is_success())
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .search(SearchParts::Index(&[&self.index_name]))
            .body(json!({
                "query": {
                    "match_all": {}
                },
                "_source": ["key"],
                "size": 10000
            }))
            .send()
            .await
            .map_err(|e| HllError::Storage(format!("Failed to list keys: {}", e)))?;

        if !response.status_code().is_success() {
            return Err(HllError::Storage(format!(
                "Elasticsearch returned status: {}",
                response.status_code()
            )));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| HllError::Storage(format!("Failed to parse response: {}", e)))?;

        let hits = body["hits"]["hits"]
            .as_array()
            .ok_or_else(|| HllError::Storage("Invalid response format".to_string()))?;

        let keys = hits
            .iter()
            .filter_map(|hit| hit["_source"]["key"].as_str())
            .map(String::from)
            .collect();

        Ok(keys)
    }
}
