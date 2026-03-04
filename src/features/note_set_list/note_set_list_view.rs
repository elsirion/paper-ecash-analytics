use std::collections::HashSet;

use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;

use crate::api::ObserverClient;
use crate::components::{Button, ButtonVariant, EmptyState};
use crate::features::import::CreateNoteSetModal;
use crate::state::{use_app_state, ToastVariant};

use super::NoteSetCard;

#[component]
pub fn NoteSetListView() -> impl IntoView {
    let state = use_app_state();
    let create_modal_open = RwSignal::new(false);
    let refreshing_sets = RwSignal::new(HashSet::<Uuid>::new());

    view! {
        <div class="max-w-6xl mx-auto p-4">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                    "Note Sets"
                </h1>
                <Button
                    variant=ButtonVariant::Primary
                    on_click=Callback::new(move |_| create_modal_open.set(true))
                >
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                    "New Note Set"
                </Button>
            </div>

            <Show
                when=move || !state.note_sets.get().is_empty()
                fallback=move || view! {
                    <EmptyState
                        title="No note sets yet"
                        description="Create a note set to start tracking ecash notes."
                    >
                        <Button
                            variant=ButtonVariant::Primary
                            on_click=Callback::new(move |_| create_modal_open.set(true))
                        >
                            "New Note Set"
                        </Button>
                    </EmptyState>
                }
            >
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <For
                        each=move || state.note_sets.get()
                        key=|set| set.id
                        children=move |note_set| {
                            let set_id = note_set.id;
                            let is_refreshing = Signal::derive(move || {
                                refreshing_sets.get().contains(&set_id)
                            });

                            let handle_refresh = move |_: ()| {
                                refreshing_sets.update(|s| { s.insert(set_id); });

                                spawn_local(async move {
                                    do_refresh(state, set_id, refreshing_sets).await;
                                });
                            };

                            view! {
                                <NoteSetCard
                                    note_set=note_set
                                    on_refresh=Callback::new(handle_refresh)
                                    is_refreshing=is_refreshing
                                />
                            }
                        }
                    />
                </div>
            </Show>

            <CreateNoteSetModal
                open=Signal::derive(move || create_modal_open.get())
                on_close=Callback::new(move |_| create_modal_open.set(false))
            />
        </div>
    }
}

async fn do_refresh(
    state: crate::state::AppState,
    set_id: Uuid,
    refreshing_sets: RwSignal<HashSet<Uuid>>,
) {
    let note_set = match state.get_note_set(set_id) {
        Some(s) => s,
        None => {
            refreshing_sets.update(|s| { s.remove(&set_id); });
            return;
        }
    };

    let api_url = state.settings.get_untracked().api_url;
    let client = ObserverClient::new(api_url);

    let unspent_nonces: Vec<String> = note_set
        .unspent_nonces()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    if unspent_nonces.is_empty() {
        state.add_toast("No unspent notes to check".to_string(), ToastVariant::Info);
        refreshing_sets.update(|s| { s.remove(&set_id); });
        return;
    }

    match client
        .check_spend_status(&note_set.federation_id, unspent_nonces.clone())
        .await
    {
        Ok(spent_results) => {
            let spent_count = spent_results.len();

            if !spent_results.is_empty() {
                state.mark_notes_spent(set_id, spent_results);
            }

            let unspent_refs: Vec<&str> = unspent_nonces.iter().map(|s| s.as_str()).collect();
            state.mark_notes_checked(set_id, &unspent_refs);

            if spent_count > 0 {
                state.add_toast(
                    format!("{} notes marked as spent", spent_count),
                    ToastVariant::Success,
                );
            } else {
                state.add_toast("All notes still unspent".to_string(), ToastVariant::Info);
            }
        }
        Err(e) => {
            state.add_toast(format!("Refresh failed: {}", e), ToastVariant::Error);
        }
    }

    refreshing_sets.update(|s| { s.remove(&set_id); });
}
