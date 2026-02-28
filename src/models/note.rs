use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpendInfo {
    pub session_index: u64,
    pub estimated_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoteStatus {
    Unspent,
    Spent(SpendInfo),
    Error(String),
}

impl NoteStatus {
    pub fn is_spent(&self) -> bool {
        matches!(self, NoteStatus::Spent(_))
    }

    pub fn is_unspent(&self) -> bool {
        matches!(self, NoteStatus::Unspent)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, NoteStatus::Error(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub nonce: String,
    pub amount_msat: u64,
    pub status: NoteStatus,
    pub last_checked: Option<DateTime<Utc>>,
    /// Index in the original import order (0-based)
    #[serde(default)]
    pub index: usize,
}

impl Note {
    pub fn new(nonce: String, amount_msat: u64) -> Self {
        Self {
            nonce,
            amount_msat,
            status: NoteStatus::Unspent,
            last_checked: None,
            index: 0,
        }
    }

    pub fn with_index(nonce: String, amount_msat: u64, index: usize) -> Self {
        Self {
            nonce,
            amount_msat,
            status: NoteStatus::Unspent,
            last_checked: None,
            index,
        }
    }

    /// Get redemption timestamp if spent
    pub fn redemption_time(&self) -> Option<DateTime<Utc>> {
        match &self.status {
            NoteStatus::Spent(info) => info.estimated_timestamp,
            _ => None,
        }
    }

    pub fn amount_sats(&self) -> u64 {
        self.amount_msat / 1000
    }

    pub fn mark_spent(&mut self, info: SpendInfo) {
        self.status = NoteStatus::Spent(info);
        self.last_checked = Some(Utc::now());
    }

    pub fn mark_checked(&mut self) {
        self.last_checked = Some(Utc::now());
    }

    pub fn mark_error(&mut self, error: String) {
        self.status = NoteStatus::Error(error);
        self.last_checked = Some(Utc::now());
    }
}
