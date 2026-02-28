use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct SpendCheckRequest {
    pub nonces: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpendInfoResponse {
    pub session_index: u64,
    pub estimated_timestamp: Option<DateTime<Utc>>,
}

/// Response from the spend check API
/// Maps nonce -> spend info (only spent nonces are included)
pub type SpendCheckResponse = HashMap<String, SpendInfoResponse>;
