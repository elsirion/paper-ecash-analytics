use leptos::prelude::GetUntracked;
use uuid::Uuid;

use crate::api::ObserverClient;
use crate::state::{AppState, ToastVariant};

pub async fn refresh_after_import(state: AppState, set_id: Uuid) {
    let Some(note_set) = state.get_note_set(set_id) else {
        return;
    };

    if note_set.federation_id.is_empty() {
        return;
    }

    let unspent_nonces: Vec<String> = note_set
        .unspent_nonces()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    if unspent_nonces.is_empty() {
        return;
    }

    let api_url = state.settings.get_untracked().api_url;
    let client = ObserverClient::new(api_url);

    match client
        .check_spend_status(&note_set.federation_id, unspent_nonces.clone())
        .await
    {
        Ok(spent_results) => {
            let spent_count = spent_results.len();

            if spent_count > 0 {
                state.mark_notes_spent(set_id, spent_results);
                state.add_toast(
                    format!("Auto-refresh: {} notes marked as spent", spent_count),
                    ToastVariant::Success,
                );
            }

            let unspent_refs: Vec<&str> = unspent_nonces.iter().map(|s| s.as_str()).collect();
            state.mark_notes_checked(set_id, &unspent_refs);
        }
        Err(e) => {
            state.add_toast(format!("Auto-refresh failed: {}", e), ToastVariant::Error);
        }
    }
}
