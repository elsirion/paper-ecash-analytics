use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{FileReader, HtmlInputElement};

use crate::components::{Button, ButtonVariant, Modal};
use crate::models::{parse_csv_notes, parse_oob_notes, Note};
use crate::state::{use_app_state, ToastVariant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ImportTab {
    Paste,
    Csv,
}

#[component]
pub fn ImportNotesModal(
    set_id: Uuid,
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let state = use_app_state();

    let active_tab = RwSignal::new(ImportTab::Csv);
    let ecash_input = RwSignal::new(String::new());
    let csv_content = RwSignal::new(String::new());
    let csv_filename = RwSignal::new(String::new());
    let is_importing = RwSignal::new(false);
    let error_message = RwSignal::new(Option::<String>::None);

    let schedule_close = move || {
        ecash_input.set(String::new());
        csv_content.set(String::new());
        csv_filename.set(String::new());
        error_message.set(None);
        is_importing.set(false);
        active_tab.set(ImportTab::Csv);
        on_close.run(());
    };

    let validate_and_add = move |notes: Vec<Note>, federation_id: String| {
        let current_fed_id = state
            .get_note_set(set_id)
            .map(|s| s.federation_id.clone())
            .unwrap_or_default();

        if !current_fed_id.is_empty() && current_fed_id != federation_id {
            error_message.set(Some(
                "Federation ID mismatch: these notes belong to a different federation".to_string(),
            ));
            is_importing.set(false);
            return;
        }

        let count = notes.len();
        state.add_notes_to_set(set_id, notes, federation_id);
        state.add_toast(
            format!("Added {} notes", count),
            ToastVariant::Success,
        );
        schedule_close();
    };

    let handle_paste_import = move |_| {
        let input = ecash_input.get();
        if input.trim().is_empty() {
            error_message.set(Some("Please enter an ecash string".to_string()));
            return;
        }

        is_importing.set(true);
        error_message.set(None);

        match parse_oob_notes(&input) {
            Ok(parsed) => {
                validate_and_add(parsed.notes, parsed.federation_id);
            }
            Err(e) => {
                error_message.set(Some(format!("Failed to parse ecash: {}", e)));
                is_importing.set(false);
            }
        }
    };

    let handle_csv_import = move |_| {
        let content = csv_content.get();
        if content.trim().is_empty() {
            error_message.set(Some("Please select a CSV file".to_string()));
            return;
        }

        is_importing.set(true);
        error_message.set(None);

        let results = parse_csv_notes(&content);
        let mut all_notes: Vec<Note> = Vec::new();
        let mut federation_id: Option<String> = None;
        let mut errors: Vec<String> = Vec::new();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(parsed) => {
                    if federation_id.is_none() {
                        federation_id = Some(parsed.federation_id.clone());
                    } else if federation_id.as_ref() != Some(&parsed.federation_id) {
                        errors.push(format!("Line {}: Different federation ID", i + 1));
                        continue;
                    }
                    all_notes.extend(parsed.notes);
                }
                Err(e) => {
                    errors.push(format!("Line {}: {}", i + 1, e));
                }
            }
        }

        if all_notes.is_empty() {
            let error_msg = if errors.is_empty() {
                "No valid notes found in CSV".to_string()
            } else {
                errors.join("\n")
            };
            error_message.set(Some(error_msg));
            is_importing.set(false);
            return;
        }

        let fed_id = federation_id.unwrap_or_default();

        if !errors.is_empty() {
            state.add_toast(
                format!("Imported with {} errors", errors.len()),
                ToastVariant::Warning,
            );
        }

        validate_and_add(all_notes, fed_id);
    };

    let handle_file_select = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input: HtmlInputElement = target.unchecked_into();

        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                csv_filename.set(file.name());

                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();

                let onload =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                        if let Ok(result) = reader_clone.result() {
                            if let Some(text) = result.as_string() {
                                csv_content.set(text);
                            }
                        }
                    }) as Box<dyn FnMut(_)>);

                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();

                let _ = reader.read_as_text(&file);
            }
        }
    };

    view! {
        <Modal open=open on_close=Callback::new(move |()| schedule_close()) title="Import Notes">
            // Tab buttons
            <div class="flex border-b border-gray-200 dark:border-gray-700 mb-4">
                <button
                    class=move || {
                        format!(
                            "px-4 py-2 text-sm font-medium border-b-2 -mb-px {}",
                            if active_tab.get() == ImportTab::Csv {
                                "border-primary-600 text-primary-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400"
                            }
                        )
                    }
                    on:click=move |_| active_tab.set(ImportTab::Csv)
                >
                    "CSV File"
                </button>
                <button
                    class=move || {
                        format!(
                            "px-4 py-2 text-sm font-medium border-b-2 -mb-px {}",
                            if active_tab.get() == ImportTab::Paste {
                                "border-primary-600 text-primary-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400"
                            }
                        )
                    }
                    on:click=move |_| active_tab.set(ImportTab::Paste)
                >
                    "Paste"
                </button>
            </div>

            // Error message
            {move || error_message.get().map(|msg| view! {
                <div class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-lg text-sm">
                    {msg}
                </div>
            })}

            // Paste tab content
            <Show when=move || active_tab.get() == ImportTab::Paste>
                <div class="mb-4">
                    <label class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                        "Ecash String"
                    </label>
                    <textarea
                        class="input resize-none font-mono text-xs"
                        rows="6"
                        placeholder="fedimint1..."
                        prop:value=move || ecash_input.get()
                        on:input=move |ev| ecash_input.set(event_target_value(&ev))
                    ></textarea>
                </div>

                <div class="flex justify-end gap-2">
                    <Button
                        variant=ButtonVariant::Outline
                        on_click=Callback::new(move |_| schedule_close())
                    >
                        "Cancel"
                    </Button>
                    <Button
                        variant=ButtonVariant::Primary
                        loading=Signal::derive(move || is_importing.get())
                        on_click=Callback::new(handle_paste_import)
                    >
                        "Import"
                    </Button>
                </div>
            </Show>

            // CSV tab content
            <Show when=move || active_tab.get() == ImportTab::Csv>
                <div class="mb-4">
                    <label class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                        "CSV File"
                    </label>
                    <div class="flex items-center justify-center w-full">
                        <label class="flex flex-col items-center justify-center w-full h-32 border-2 border-gray-300 border-dashed rounded-lg cursor-pointer bg-gray-50 dark:hover:bg-gray-800 dark:bg-gray-700 hover:bg-gray-100 dark:border-gray-600">
                            <div class="flex flex-col items-center justify-center pt-5 pb-6">
                                <Show
                                    when=move || csv_filename.get().is_empty()
                                    fallback=move || view! {
                                        <svg class="w-8 h-8 mb-2 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                        <p class="text-sm text-gray-700 dark:text-gray-300">{move || csv_filename.get()}</p>
                                    }
                                >
                                    <svg class="w-8 h-8 mb-2 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                                    </svg>
                                    <p class="mb-2 text-sm text-gray-500 dark:text-gray-400">
                                        <span class="font-semibold">"Click to upload"</span>
                                    </p>
                                    <p class="text-xs text-gray-500 dark:text-gray-400">"CSV with one ecash string per line"</p>
                                </Show>
                            </div>
                            <input
                                type="file"
                                class="hidden"
                                accept=".csv,.txt"
                                on:change=handle_file_select
                            />
                        </label>
                    </div>
                </div>

                <div class="flex justify-end gap-2">
                    <Button
                        variant=ButtonVariant::Outline
                        on_click=Callback::new(move |_| schedule_close())
                    >
                        "Cancel"
                    </Button>
                    <Button
                        variant=ButtonVariant::Primary
                        loading=Signal::derive(move || is_importing.get())
                        disabled=Signal::derive(move || csv_content.get().is_empty())
                        on_click=Callback::new(handle_csv_import)
                    >
                        "Import"
                    </Button>
                </div>
            </Show>
        </Modal>
    }
}
