use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;
use wasm_bindgen::prelude::*;

mod api;
mod components;
mod features;
mod models;
mod state;
mod utils;

use components::ToastContainer;
use features::note_set_detail::NoteSetDetailView;
use features::note_set_list::NoteSetListView;
use features::settings::SettingsView;
use state::provide_app_state;

#[component]
fn Header() -> impl IntoView {
    let state = state::use_app_state();

    let toggle_dark_mode = move |_: web_sys::MouseEvent| {
        state.update_settings(|s| {
            s.dark_mode = !s.dark_mode;
        });

        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html) = document.document_element() {
                if state.settings.get().dark_mode {
                    let _ = html.class_list().add_1("dark");
                } else {
                    let _ = html.class_list().remove_1("dark");
                }
            }
        }
    };

    view! {
        <header class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
            <div class="max-w-6xl mx-auto px-4 py-3 flex items-center justify-between">
                <A href="/" attr:class="flex items-center gap-2 text-gray-900 dark:text-white hover:text-primary-600 dark:hover:text-primary-400">
                    <svg class="w-8 h-8 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <span class="text-xl font-bold">"Paper Ecash Analytics"</span>
                </A>

                <div class="flex items-center gap-2">
                    <button
                        class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                        on:click=toggle_dark_mode
                        title="Toggle dark mode"
                    >
                        <Show
                            when=move || state.settings.get().dark_mode
                            fallback=|| view! {
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                                </svg>
                            }
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
                            </svg>
                        </Show>
                    </button>

                    <A
                        href="/settings"
                        attr:class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
                        attr:title="Settings"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                    </A>
                </div>
            </div>
        </header>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_app_state();

    // Apply initial dark mode setting
    Effect::new(move || {
        let state = state::use_app_state();
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html) = document.document_element() {
                if state.settings.get_untracked().dark_mode {
                    let _ = html.class_list().add_1("dark");
                }
            }
        }
    });

    view! {
        <Router>
            <Header />
            <main class="min-h-[calc(100vh-64px)]">
                <Routes fallback=|| view! { <p class="p-4">"Page not found"</p> }>
                    <Route path=path!("/") view=NoteSetListView />
                    <Route path=path!("/set/:id") view=NoteSetDetailView />
                    <Route path=path!("/settings") view=SettingsView />
                </Routes>
            </main>
            <ToastContainer />
        </Router>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");

    log::info!("Starting Paper Ecash Analytics...");

    leptos::mount::mount_to_body(App);
}
