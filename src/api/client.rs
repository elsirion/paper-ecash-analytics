use gloo_net::http::Request;
use thiserror::Error;

use super::types::{FederationListEntry, FederationMeta, SpendCheckRequest, SpendCheckResponse};
use crate::models::SpendInfo;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Failed to parse response: {0}")]
    Parse(String),
    #[error("API error: {0}")]
    Api(String),
}

pub struct ObserverClient {
    base_url: String,
}

impl ObserverClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// Check which nonces have been spent
    /// Returns a list of (nonce, SpendInfo) for spent nonces only
    pub async fn check_spend_status(
        &self,
        federation_id: &str,
        nonces: Vec<String>,
    ) -> Result<Vec<(String, SpendInfo)>, ApiError> {
        if nonces.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!(
            "{}/federations/{}/nonces/spend",
            self.base_url, federation_id
        );

        let request = SpendCheckRequest { nonces };

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .map_err(|e| ApiError::Parse(e.to_string()))?
            .send()
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                // Check for CORS-related errors
                if err_str.contains("NetworkError") || err_str.contains("Failed to fetch") {
                    ApiError::Network("Request failed - the observer API may not support browser requests (CORS). Try using a CORS proxy.".to_string())
                } else {
                    ApiError::Network(err_str)
                }
            })?;

        if !response.ok() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::Api(format!("HTTP {}: {}", status, text)));
        }

        let spend_response: SpendCheckResponse = response
            .json()
            .await
            .map_err(|e| ApiError::Parse(e.to_string()))?;

        let results = spend_response
            .into_iter()
            .map(|(nonce, info)| {
                (
                    nonce,
                    SpendInfo {
                        session_index: info.session_index,
                        estimated_timestamp: info.estimated_timestamp,
                    },
                )
            })
            .collect();

        Ok(results)
    }

    /// Fetch list of federation IDs known to the observer
    pub async fn fetch_federations(&self) -> Result<Vec<String>, ApiError> {
        let url = format!("{}/federations", self.base_url);

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.ok() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::Api(format!("HTTP {}: {}", status, text)));
        }

        let entries: Vec<FederationListEntry> = response
            .json()
            .await
            .map_err(|e| ApiError::Parse(e.to_string()))?;

        Ok(entries.into_iter().map(|e| e.id).collect())
    }

    /// Fetch federation metadata (name, etc.)
    pub async fn fetch_federation_meta(
        &self,
        federation_id: &str,
    ) -> Result<FederationMeta, ApiError> {
        let url = format!(
            "{}/federations/{}/meta",
            self.base_url, federation_id
        );

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.ok() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiError::Api(format!("HTTP {}: {}", status, text)));
        }

        response
            .json()
            .await
            .map_err(|e| ApiError::Parse(e.to_string()))
    }
}

impl Default for ObserverClient {
    fn default() -> Self {
        Self::new("https://observer.fedimint.org/api".to_string())
    }
}
