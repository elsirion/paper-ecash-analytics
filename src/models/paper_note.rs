use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::note::{Note, NoteStatus, SpendInfo};

#[derive(Debug, Clone, PartialEq)]
pub enum PaperNoteStatus {
    Unspent,
    Spent,
    PartiallySpent,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaperNote {
    pub paper_note_id: Uuid,
    pub ecash_notes: Vec<Note>,
    /// Lowest index among constituent ecash notes (used for import-order sorting)
    pub index: usize,
}

impl PaperNote {
    pub fn total_amount_msat(&self) -> u64 {
        self.ecash_notes.iter().map(|n| n.amount_msat).sum()
    }

    pub fn status(&self) -> PaperNoteStatus {
        let all_spent = self.ecash_notes.iter().all(|n| n.status.is_spent());
        let all_unspent = self.ecash_notes.iter().all(|n| n.status.is_unspent());
        let has_error = self.ecash_notes.iter().any(|n| n.status.is_error());

        if has_error {
            PaperNoteStatus::Error
        } else if all_spent {
            PaperNoteStatus::Spent
        } else if all_unspent {
            PaperNoteStatus::Unspent
        } else {
            PaperNoteStatus::PartiallySpent
        }
    }

    pub fn is_spent(&self) -> bool {
        self.status() == PaperNoteStatus::Spent
    }

    pub fn is_unspent(&self) -> bool {
        self.status() == PaperNoteStatus::Unspent
    }

    /// Returns the latest redemption timestamp among constituent ecash notes
    pub fn redemption_time(&self) -> Option<DateTime<Utc>> {
        self.ecash_notes
            .iter()
            .filter_map(|n| n.redemption_time())
            .max()
    }

    pub fn last_checked(&self) -> Option<DateTime<Utc>> {
        self.ecash_notes
            .iter()
            .filter_map(|n| n.last_checked)
            .max()
    }

    /// Returns spend info from the first spent ecash note (for session display)
    pub fn spend_info(&self) -> Option<&SpendInfo> {
        self.ecash_notes.iter().find_map(|n| match &n.status {
            NoteStatus::Spent(info) => Some(info),
            _ => None,
        })
    }

    /// Display nonce: first ecash note's nonce (truncated)
    pub fn display_nonce(&self) -> &str {
        self.ecash_notes
            .first()
            .map(|n| n.nonce.as_str())
            .unwrap_or("")
    }

    /// Returns a sorted list of (amount_msat, count) pairs
    pub fn denomination_breakdown(&self) -> Vec<(u64, usize)> {
        let mut counts: BTreeMap<u64, usize> = BTreeMap::new();
        for note in &self.ecash_notes {
            *counts.entry(note.amount_msat).or_default() += 1;
        }
        counts.into_iter().collect()
    }

    pub fn ecash_note_count(&self) -> usize {
        self.ecash_notes.len()
    }
}

/// Group flat ecash notes into paper notes by paper_note_id
pub fn group_into_paper_notes(notes: &[Note]) -> Vec<PaperNote> {
    let mut groups: BTreeMap<Uuid, Vec<Note>> = BTreeMap::new();
    for note in notes {
        groups
            .entry(note.paper_note_id)
            .or_default()
            .push(note.clone());
    }
    groups
        .into_iter()
        .map(|(paper_note_id, ecash_notes)| {
            let index = ecash_notes.iter().map(|n| n.index).min().unwrap_or(0);
            PaperNote {
                paper_note_id,
                ecash_notes,
                index,
            }
        })
        .collect()
}
