use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Note, NoteSet, NoteStatus, SpendInfo};
use super::persistence;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub api_url: String,
    pub dark_mode: bool,
    pub default_auto_refresh_interval: Option<u64>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_url: "https://observer.fedimint.org/api".to_string(),
            dark_mode: false,
            default_auto_refresh_interval: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub note_sets: RwSignal<Vec<NoteSet>>,
    pub settings: RwSignal<Settings>,
    pub toasts: RwSignal<Vec<Toast>>,
    pub known_federations: RwSignal<Option<Vec<String>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: Uuid,
    pub message: String,
    pub variant: ToastVariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToastVariant {
    Success,
    Error,
    Info,
    Warning,
}

impl AppState {
    pub fn new() -> Self {
        let note_sets = persistence::load_note_sets().unwrap_or_default();
        let settings = persistence::load_settings().unwrap_or_default();

        Self {
            note_sets: RwSignal::new(note_sets),
            settings: RwSignal::new(settings),
            toasts: RwSignal::new(Vec::new()),
            known_federations: RwSignal::new(None),
        }
    }

    pub fn add_note_set(&self, note_set: NoteSet) {
        self.note_sets.update(|sets| {
            sets.push(note_set);
        });
        self.persist_note_sets();
    }

    pub fn remove_note_set(&self, id: Uuid) {
        self.note_sets.update(|sets| {
            sets.retain(|s| s.id != id);
        });
        self.persist_note_sets();
    }

    pub fn get_note_set(&self, id: Uuid) -> Option<NoteSet> {
        self.note_sets.get_untracked().into_iter().find(|s| s.id == id)
    }

    #[allow(dead_code)]
    pub fn update_note_set<F>(&self, id: Uuid, f: F)
    where
        F: FnOnce(&mut NoteSet),
    {
        self.note_sets.update(|sets| {
            if let Some(set) = sets.iter_mut().find(|s| s.id == id) {
                f(set);
            }
        });
        self.persist_note_sets();
    }

    #[allow(dead_code)]
    pub fn update_note_status(&self, set_id: Uuid, nonce: &str, status: NoteStatus) {
        self.note_sets.update(|sets| {
            if let Some(set) = sets.iter_mut().find(|s| s.id == set_id) {
                set.update_note_status(nonce, status);
            }
        });
        self.persist_note_sets();
    }

    pub fn mark_notes_spent(&self, set_id: Uuid, spend_results: Vec<(String, SpendInfo)>) {
        self.note_sets.update(|sets| {
            if let Some(set) = sets.iter_mut().find(|s| s.id == set_id) {
                for (nonce, info) in spend_results {
                    set.update_note_status(&nonce, NoteStatus::Spent(info));
                }
                set.mark_refreshed();
            }
        });
        self.persist_note_sets();
    }

    pub fn mark_notes_checked(&self, set_id: Uuid, nonces: &[&str]) {
        self.note_sets.update(|sets| {
            if let Some(set) = sets.iter_mut().find(|s| s.id == set_id) {
                for note in &mut set.notes {
                    if nonces.contains(&note.nonce.as_str()) && note.status.is_unspent() {
                        note.mark_checked();
                    }
                }
                set.mark_refreshed();
            }
        });
        self.persist_note_sets();
    }

    /// Add notes to an existing set. Returns the number of distinct paper notes added.
    /// Returns Err if the federation_id doesn't match or isn't known to the observer.
    pub fn add_notes_to_set(
        &self,
        set_id: Uuid,
        new_notes: Vec<Note>,
        federation_id: String,
    ) -> Result<usize, String> {
        if let Some(known) = self.known_federations.get_untracked() {
            if !known.iter().any(|f| *f == federation_id) {
                return Err(format!(
                    "Federation {} is not observed by the configured observer",
                    &federation_id[..16.min(federation_id.len())]
                ));
            }
        }
        let count = new_notes
            .iter()
            .map(|n| n.paper_note_id)
            .collect::<std::collections::HashSet<_>>()
            .len();
        let mut error = None;
        self.note_sets.update(|sets| {
            if let Some(set) = sets.iter_mut().find(|s| s.id == set_id) {
                if set.federation_id.is_empty() {
                    set.federation_id = federation_id;
                } else if set.federation_id != federation_id {
                    error = Some(format!(
                        "Federation ID mismatch: expected {}, got {}",
                        &set.federation_id[..16.min(set.federation_id.len())],
                        &federation_id[..16.min(federation_id.len())]
                    ));
                    return;
                }
                let start_index = set.notes.len();
                for (i, mut note) in new_notes.into_iter().enumerate() {
                    note.index = start_index + i;
                    set.notes.push(note);
                }
            }
        });
        if let Some(e) = error {
            return Err(e);
        }
        self.persist_note_sets();
        Ok(count)
    }

    pub fn update_settings<F>(&self, f: F)
    where
        F: FnOnce(&mut Settings),
    {
        self.settings.update(f);
        self.persist_settings();
    }

    pub fn add_toast(&self, message: String, variant: ToastVariant) {
        let toast = Toast {
            id: Uuid::new_v4(),
            message,
            variant,
        };
        self.toasts.update(|t| t.push(toast));
    }

    pub fn remove_toast(&self, id: Uuid) {
        self.toasts.update(|t| t.retain(|toast| toast.id != id));
    }

    fn persist_note_sets(&self) {
        let sets = self.note_sets.get_untracked();
        if let Err(e) = persistence::save_note_sets(&sets) {
            log::error!("Failed to save note sets: {}", e);
        }
    }

    fn persist_settings(&self) {
        let settings = self.settings.get_untracked();
        if let Err(e) = persistence::save_settings(&settings) {
            log::error!("Failed to save settings: {}", e);
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn provide_app_state() {
    let state = AppState::new();
    provide_context(state);
}

pub fn use_app_state() -> AppState {
    expect_context::<AppState>()
}
