use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::{Badge, BadgeVariant, Spinner};
use crate::models::NoteSet;
use crate::utils::encoding::format_amount_msat;
use crate::utils::time::format_relative_time;

#[component]
pub fn NoteSetCard(
    note_set: NoteSet,
    #[prop(into)] on_refresh: Callback<()>,
    #[prop(into)] is_refreshing: Signal<bool>,
) -> impl IntoView {
    let navigate = use_navigate();
    let set_id = note_set.id;

    let handle_click = move |_| {
        navigate(&format!("/set/{}", set_id), Default::default());
    };

    let total_sats = note_set.total_amount_msat();
    let spent_sats = note_set.spent_amount_msat();
    let unspent_sats = note_set.unspent_amount_msat();

    let spent_count = note_set.spent_count();
    let unspent_count = note_set.unspent_count();
    let total_count = note_set.note_count();

    let last_refreshed = note_set.last_refreshed;

    view! {
        <div class="card hover:shadow-lg transition-shadow cursor-pointer" on:click=handle_click>
            <div class="flex justify-between items-start mb-4">
                <div>
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {note_set.name.clone()}
                    </h3>
                    <p class="text-sm text-gray-500 dark:text-gray-400 truncate max-w-[200px]">
                        {if note_set.federation_id.is_empty() {
                            "No notes imported yet".to_string()
                        } else {
                            format!("{}...", &note_set.federation_id[..16.min(note_set.federation_id.len())])
                        }}
                    </p>
                </div>
                <button
                    class="p-2 text-gray-500 hover:text-primary-600 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        on_refresh.run(());
                    }
                    disabled=move || is_refreshing.get()
                >
                    {move || {
                        if is_refreshing.get() {
                            view! { <Spinner size="sm".to_string() /> }.into_any()
                        } else {
                            view! {
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                                </svg>
                            }.into_any()
                        }
                    }}
                </button>
            </div>

            // Stats
            <div class="grid grid-cols-3 gap-2 mb-4">
                <div class="text-center p-2 bg-gray-50 dark:bg-gray-700 rounded-lg">
                    <div class="text-xs text-gray-500 dark:text-gray-400">"Total"</div>
                    <div class="text-sm font-semibold text-gray-900 dark:text-white">
                        {format_amount_msat(total_sats)}
                    </div>
                </div>
                <div class="text-center p-2 bg-green-50 dark:bg-green-900/20 rounded-lg">
                    <div class="text-xs text-green-600 dark:text-green-400">"Unspent"</div>
                    <div class="text-sm font-semibold text-green-700 dark:text-green-300">
                        {format_amount_msat(unspent_sats)}
                    </div>
                </div>
                <div class="text-center p-2 bg-red-50 dark:bg-red-900/20 rounded-lg">
                    <div class="text-xs text-red-600 dark:text-red-400">"Spent"</div>
                    <div class="text-sm font-semibold text-red-700 dark:text-red-300">
                        {format_amount_msat(spent_sats)}
                    </div>
                </div>
            </div>

            // Note counts
            <div class="flex items-center gap-2 mb-3">
                <Badge variant=BadgeVariant::Gray>
                    {format!("{} notes", total_count)}
                </Badge>
                <Badge variant=BadgeVariant::Green>
                    {format!("{} unspent", unspent_count)}
                </Badge>
                <Badge variant=BadgeVariant::Red>
                    {format!("{} spent", spent_count)}
                </Badge>
            </div>

            // Last refreshed
            <div class="text-xs text-gray-500 dark:text-gray-400">
                {match last_refreshed {
                    Some(dt) => format!("Last checked: {}", format_relative_time(&dt)),
                    None => "Never checked".to_string(),
                }}
            </div>
        </div>
    }
}
