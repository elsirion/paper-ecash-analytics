use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Note, NoteStatus};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoteSet {
    pub id: Uuid,
    pub name: String,
    pub federation_id: String,
    pub notes: Vec<Note>,
    pub created_at: DateTime<Utc>,
    pub last_refreshed: Option<DateTime<Utc>>,
    pub auto_refresh_interval: Option<u64>,
}

impl NoteSet {
    #[allow(dead_code)]
    pub fn new(name: String, federation_id: String, notes: Vec<Note>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            federation_id,
            notes,
            created_at: Utc::now(),
            last_refreshed: None,
            auto_refresh_interval: None,
        }
    }

    pub fn new_empty(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            federation_id: String::new(),
            notes: Vec::new(),
            created_at: Utc::now(),
            last_refreshed: None,
            auto_refresh_interval: None,
        }
    }

    pub fn total_amount_msat(&self) -> u64 {
        self.notes.iter().map(|n| n.amount_msat).sum()
    }

    #[allow(dead_code)]
    pub fn total_amount_sats(&self) -> u64 {
        self.total_amount_msat() / 1000
    }

    pub fn spent_amount_msat(&self) -> u64 {
        self.notes
            .iter()
            .filter(|n| n.status.is_spent())
            .map(|n| n.amount_msat)
            .sum()
    }

    #[allow(dead_code)]
    pub fn spent_amount_sats(&self) -> u64 {
        self.spent_amount_msat() / 1000
    }

    pub fn unspent_amount_msat(&self) -> u64 {
        self.notes
            .iter()
            .filter(|n| n.status.is_unspent())
            .map(|n| n.amount_msat)
            .sum()
    }

    #[allow(dead_code)]
    pub fn unspent_amount_sats(&self) -> u64 {
        self.unspent_amount_msat() / 1000
    }

    pub fn note_count(&self) -> usize {
        self.notes.len()
    }

    pub fn spent_count(&self) -> usize {
        self.notes.iter().filter(|n| n.status.is_spent()).count()
    }

    pub fn unspent_count(&self) -> usize {
        self.notes.iter().filter(|n| n.status.is_unspent()).count()
    }

    #[allow(dead_code)]
    pub fn error_count(&self) -> usize {
        self.notes.iter().filter(|n| n.status.is_error()).count()
    }

    pub fn unspent_nonces(&self) -> Vec<&str> {
        self.notes
            .iter()
            .filter(|n| n.status.is_unspent())
            .map(|n| n.nonce.as_str())
            .collect()
    }

    pub fn mark_refreshed(&mut self) {
        self.last_refreshed = Some(Utc::now());
    }

    pub fn update_note_status(&mut self, nonce: &str, status: NoteStatus) {
        if let Some(note) = self.notes.iter_mut().find(|n| n.nonce == nonce) {
            note.status = status;
            note.last_checked = Some(Utc::now());
        }
    }
}
