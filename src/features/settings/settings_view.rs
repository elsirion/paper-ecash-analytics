use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::{Button, ButtonVariant, Card};
use crate::state::use_app_state;

#[component]
pub fn SettingsView() -> impl IntoView {
    let state = use_app_state();
    let navigate = use_navigate();

    let api_url = RwSignal::new(state.settings.get_untracked().api_url);
    let dark_mode = RwSignal::new(state.settings.get_untracked().dark_mode);

    let handle_save = move |_: ()| {
        state.update_settings(|s| {
            s.api_url = api_url.get();
            s.dark_mode = dark_mode.get();
        });

        // Apply dark mode
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html) = document.document_element() {
                if dark_mode.get() {
                    let _ = html.class_list().add_1("dark");
                } else {
                    let _ = html.class_list().remove_1("dark");
                }
            }
        }
    };

    let handle_back = {
        let navigate = navigate.clone();
        move |_: web_sys::MouseEvent| {
            navigate("/", Default::default());
        }
    };

    view! {
        <div class="max-w-2xl mx-auto p-4">
            <div class="flex items-center gap-4 mb-6">
                <button
                    class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg"
                    on:click=handle_back
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                    </svg>
                </button>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                    "Settings"
                </h1>
            </div>

            <Card class="mb-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                    "API Configuration"
                </h2>

                <div class="mb-4">
                    <label class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                        "Observer API URL"
                    </label>
                    <input
                        type="text"
                        class="input"
                        placeholder="https://observer.fedimint.org/api"
                        prop:value=move || api_url.get()
                        on:input=move |ev| api_url.set(event_target_value(&ev))
                    />
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                        "The base URL for the fedimint observer API"
                    </p>
                </div>
            </Card>

            <Card class="mb-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                    "Appearance"
                </h2>

                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-medium text-gray-900 dark:text-white">
                            "Dark Mode"
                        </div>
                        <p class="text-sm text-gray-500 dark:text-gray-400">
                            "Use dark color scheme"
                        </p>
                    </div>
                    <button
                        class=move || {
                            format!(
                                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors {}",
                                if dark_mode.get() { "bg-primary-600" } else { "bg-gray-200 dark:bg-gray-700" }
                            )
                        }
                        on:click=move |_| dark_mode.update(|v| *v = !*v)
                    >
                        <span
                            class=move || {
                                format!(
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform {}",
                                    if dark_mode.get() { "translate-x-6" } else { "translate-x-1" }
                                )
                            }
                        />
                    </button>
                </div>
            </Card>

            <Card class="mb-6">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                    "Data"
                </h2>

                <div class="space-y-4">
                    <div class="flex items-center justify-between">
                        <div>
                            <div class="text-sm font-medium text-gray-900 dark:text-white">
                                "Note Sets"
                            </div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                {move || format!("{} sets stored locally", state.note_sets.get().len())}
                            </p>
                        </div>
                    </div>
                </div>
            </Card>

            <div class="flex justify-end">
                <Button
                    variant=ButtonVariant::Primary
                    on_click=Callback::new(handle_save)
                >
                    "Save Settings"
                </Button>
            </div>
        </div>
    }
}
