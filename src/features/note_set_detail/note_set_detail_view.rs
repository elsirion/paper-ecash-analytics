use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;

use crate::api::ObserverClient;
use crate::components::{Button, ButtonVariant, Card, EmptyState, RedemptionChart};
use crate::models::Note;
use crate::state::{use_app_state, ToastVariant};
use crate::utils::encoding::format_amount_msat;
use crate::utils::time::format_relative_time;

use super::NoteRow;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SortOrder {
    #[default]
    ImportOrder,
    RedemptionDateAsc,
    RedemptionDateDesc,
}

impl SortOrder {
    fn sort_notes(&self, notes: &mut [Note]) {
        match self {
            SortOrder::ImportOrder => {
                notes.sort_by_key(|n| n.index);
            }
            SortOrder::RedemptionDateAsc => {
                notes.sort_by(|a, b| {
                    let a_time = a.redemption_time();
                    let b_time = b.redemption_time();
                    match (a_time, b_time) {
                        (Some(a), Some(b)) => a.cmp(&b),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.index.cmp(&b.index),
                    }
                });
            }
            SortOrder::RedemptionDateDesc => {
                notes.sort_by(|a, b| {
                    let a_time = a.redemption_time();
                    let b_time = b.redemption_time();
                    match (a_time, b_time) {
                        (Some(a), Some(b)) => b.cmp(&a),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.index.cmp(&b.index),
                    }
                });
            }
        }
    }
}

#[component]
pub fn NoteSetDetailView() -> impl IntoView {
    let state = use_app_state();
    let params = use_params_map();

    let is_refreshing = RwSignal::new(false);
    let show_delete_confirm = RwSignal::new(false);
    let sort_order = RwSignal::new(SortOrder::default());

    let set_id = Memo::new(move |_| {
        params.get().get("id").and_then(|id| Uuid::parse_str(&id).ok())
    });

    let note_set = Memo::new(move |_| {
        set_id.get().and_then(|id| state.get_note_set(id))
    });

    view! {
        <div class="max-w-6xl mx-auto p-4">
            <Show
                when=move || note_set.get().is_some()
                fallback=|| view! { <NotFoundView /> }
            >
                <NoteSetContent
                    state=state
                    set_id=set_id
                    note_set=note_set
                    is_refreshing=is_refreshing
                    show_delete_confirm=show_delete_confirm
                    sort_order=sort_order
                />
            </Show>
        </div>
    }
}

#[component]
fn NotFoundView() -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <EmptyState
            title="Note set not found"
            description="The note set you're looking for doesn't exist or has been deleted."
        >
            <Button
                variant=ButtonVariant::Primary
                on_click=Callback::new(move |_| {
                    navigate("/", Default::default());
                })
            >
                "Go Back"
            </Button>
        </EmptyState>
    }
}

#[component]
fn NoteSetContent(
    state: crate::state::AppState,
    set_id: Memo<Option<Uuid>>,
    note_set: Memo<Option<crate::models::NoteSet>>,
    is_refreshing: RwSignal<bool>,
    show_delete_confirm: RwSignal<bool>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let navigate = use_navigate();

    let handle_refresh = move |_: ()| {
        let Some(id) = set_id.get() else { return };
        let Some(current_set) = note_set.get() else { return };

        is_refreshing.set(true);

        spawn_local(async move {
            let api_url = state.settings.get_untracked().api_url;
            let client = ObserverClient::new(api_url);

            let unspent_nonces: Vec<String> = current_set
                .unspent_nonces()
                .into_iter()
                .map(|s| s.to_string())
                .collect();

            if unspent_nonces.is_empty() {
                state.add_toast("No unspent notes to check".to_string(), ToastVariant::Info);
                is_refreshing.set(false);
                return;
            }

            match client
                .check_spend_status(&current_set.federation_id, unspent_nonces.clone())
                .await
            {
                Ok(spent_results) => {
                    let spent_count = spent_results.len();

                    if !spent_results.is_empty() {
                        state.mark_notes_spent(id, spent_results);
                    }

                    let unspent_refs: Vec<&str> = unspent_nonces.iter().map(|s| s.as_str()).collect();
                    state.mark_notes_checked(id, &unspent_refs);

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

            is_refreshing.set(false);
        });
    };

    view! {
        {move || {
            let current_set = note_set.get().unwrap();
            let notes = current_set.notes.clone();
            let name = current_set.name.clone();
            let federation_id = current_set.federation_id.clone();
            let total_amount = current_set.total_amount_msat();
            let note_count = current_set.note_count();
            let unspent_amount = current_set.unspent_amount_msat();
            let unspent_count = current_set.unspent_count();
            let spent_amount = current_set.spent_amount_msat();
            let spent_count = current_set.spent_count();
            let last_refreshed = current_set.last_refreshed
                .map(|t| format_relative_time(&t))
                .unwrap_or_else(|| "Never".to_string());
            let created_at = format_relative_time(&current_set.created_at);
            let navigate = navigate.clone();

            view! {
                <div>
                    // Header
                    <div class="flex items-center gap-4 mb-6">
                        <button
                            class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg"
                            on:click={
                                let navigate = navigate.clone();
                                move |_| { navigate("/", Default::default()); }
                            }
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                            </svg>
                        </button>
                        <div class="flex-1">
                            <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                                {name}
                            </h1>
                            <p class="text-sm text-gray-500 dark:text-gray-400 font-mono truncate">
                                {federation_id}
                            </p>
                        </div>
                        <div class="flex gap-2">
                            <Button
                                variant=ButtonVariant::Outline
                                loading=Signal::derive(move || is_refreshing.get())
                                on_click=Callback::new(handle_refresh)
                            >
                                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                                </svg>
                                "Refresh"
                            </Button>
                            <Button
                                variant=ButtonVariant::Danger
                                on_click=Callback::new(move |_| show_delete_confirm.set(true))
                            >
                                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                                "Delete"
                            </Button>
                        </div>
                    </div>

                    // Redemption chart
                    <RedemptionChart note_set=Signal::derive(move || note_set.get()) />

                    // Stats cards
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
                        <Card>
                            <div class="text-sm text-gray-500 dark:text-gray-400">"Total"</div>
                            <div class="text-2xl font-bold text-gray-900 dark:text-white">
                                {format_amount_msat(total_amount)}
                            </div>
                            <div class="text-sm text-gray-500 dark:text-gray-400">
                                {format!("{} notes", note_count)}
                            </div>
                        </Card>
                        <Card class="bg-green-50 dark:bg-green-900/20">
                            <div class="text-sm text-green-600 dark:text-green-400">"Unspent"</div>
                            <div class="text-2xl font-bold text-green-700 dark:text-green-300">
                                {format_amount_msat(unspent_amount)}
                            </div>
                            <div class="text-sm text-green-600 dark:text-green-400">
                                {format!("{} notes", unspent_count)}
                            </div>
                        </Card>
                        <Card class="bg-red-50 dark:bg-red-900/20">
                            <div class="text-sm text-red-600 dark:text-red-400">"Spent"</div>
                            <div class="text-2xl font-bold text-red-700 dark:text-red-300">
                                {format_amount_msat(spent_amount)}
                            </div>
                            <div class="text-sm text-red-600 dark:text-red-400">
                                {format!("{} notes", spent_count)}
                            </div>
                        </Card>
                        <Card>
                            <div class="text-sm text-gray-500 dark:text-gray-400">"Last Checked"</div>
                            <div class="text-lg font-semibold text-gray-900 dark:text-white">
                                {last_refreshed}
                            </div>
                            <div class="text-sm text-gray-500 dark:text-gray-400">
                                {format!("Created {}", created_at)}
                            </div>
                        </Card>
                    </div>

                    // Notes table
                    <Card>
                        // Sort controls
                        <div class="flex items-center gap-2 mb-4 pb-4 border-b border-gray-200 dark:border-gray-700">
                            <span class="text-sm text-gray-500 dark:text-gray-400">"Sort by:"</span>
                            <button
                                class=move || format!(
                                    "px-3 py-1 text-sm rounded-lg transition-colors {}",
                                    if sort_order.get() == SortOrder::ImportOrder {
                                        "bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-300"
                                    } else {
                                        "bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
                                    }
                                )
                                on:click=move |_| sort_order.set(SortOrder::ImportOrder)
                            >
                                "Import Order"
                            </button>
                            <button
                                class=move || format!(
                                    "px-3 py-1 text-sm rounded-lg transition-colors {}",
                                    if sort_order.get() == SortOrder::RedemptionDateDesc {
                                        "bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-300"
                                    } else {
                                        "bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
                                    }
                                )
                                on:click=move |_| sort_order.set(SortOrder::RedemptionDateDesc)
                            >
                                "Newest Redemptions"
                            </button>
                            <button
                                class=move || format!(
                                    "px-3 py-1 text-sm rounded-lg transition-colors {}",
                                    if sort_order.get() == SortOrder::RedemptionDateAsc {
                                        "bg-primary-100 text-primary-700 dark:bg-primary-900 dark:text-primary-300"
                                    } else {
                                        "bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
                                    }
                                )
                                on:click=move |_| sort_order.set(SortOrder::RedemptionDateAsc)
                            >
                                "Oldest Redemptions"
                            </button>
                        </div>

                        <div class="overflow-x-auto">
                            <table class="w-full text-left">
                                <thead class="text-xs text-gray-700 dark:text-gray-400 uppercase bg-gray-50 dark:bg-gray-700">
                                    <tr>
                                        <th class="px-4 py-3">"#"</th>
                                        <th class="px-4 py-3">"Nonce"</th>
                                        <th class="px-4 py-3 text-right">"Amount"</th>
                                        <th class="px-4 py-3">"Status"</th>
                                        <th class="px-4 py-3">"Details"</th>
                                        <th class="px-4 py-3">"Last Checked"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let mut sorted_notes = notes.clone();
                                        sort_order.get().sort_notes(&mut sorted_notes);
                                        sorted_notes.into_iter().map(|note| {
                                            view! { <NoteRow note=note /> }
                                        }).collect::<Vec<_>>()
                                    }}
                                </tbody>
                            </table>
                        </div>
                    </Card>
                </div>

                // Delete confirmation modal
                <Show when=move || show_delete_confirm.get()>
                    <DeleteConfirmModal
                        state=state
                        set_id=set_id
                        show_delete_confirm=show_delete_confirm
                    />
                </Show>
            }
        }}
    }
}

#[component]
fn DeleteConfirmModal(
    state: crate::state::AppState,
    set_id: Memo<Option<Uuid>>,
    show_delete_confirm: RwSignal<bool>,
) -> impl IntoView {
    let navigate = use_navigate();

    let handle_delete = move |_: ()| {
        if let Some(id) = set_id.get() {
            state.remove_note_set(id);
            state.add_toast("Note set deleted".to_string(), ToastVariant::Success);
            navigate("/", Default::default());
        }
    };

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
            <Card class="max-w-md mx-4">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                    "Delete Note Set?"
                </h3>
                <p class="text-gray-600 dark:text-gray-400 mb-4">
                    "This will permanently delete this note set and all its data. This action cannot be undone."
                </p>
                <div class="flex justify-end gap-2">
                    <Button
                        variant=ButtonVariant::Outline
                        on_click=Callback::new(move |_| show_delete_confirm.set(false))
                    >
                        "Cancel"
                    </Button>
                    <Button
                        variant=ButtonVariant::Danger
                        on_click=Callback::new(handle_delete)
                    >
                        "Delete"
                    </Button>
                </div>
            </Card>
        </div>
    }
}
