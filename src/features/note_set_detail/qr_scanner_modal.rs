use std::collections::HashSet;

use leptos::prelude::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::components::{Button, ButtonVariant, Modal};
use crate::models::parse_oob_notes;
use crate::state::{use_app_state, ToastVariant};
use crate::utils::qr_scanner;
use super::auto_refresh::refresh_after_import;

#[component]
pub fn QrScannerModal(
    set_id: Uuid,
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let state = use_app_state();
    let scanner_error = RwSignal::new(Option::<String>::None);
    let scanner_active = RwSignal::new(false);
    let scanned_paper_ids = RwSignal::new(HashSet::<Uuid>::new());
    let imported_notes = RwSignal::new(false);

    // Store element_id in a signal so closures can be Copy
    let element_id = StoredValue::new(format!("qr-reader-{}", set_id.as_simple()));

    let stop_and_cleanup = move || {
        element_id.with_value(|id| qr_scanner::stop_qr_scanner(id));
        scanner_active.set(false);
        scanned_paper_ids.set(HashSet::new());
        scanner_error.set(None);
    };

    let handle_close = move || {
        let should_auto_refresh = imported_notes.get_untracked();
        stop_and_cleanup();
        if should_auto_refresh {
            imported_notes.set(false);
            spawn_local(async move {
                refresh_after_import(state, set_id).await;
            });
        }
        on_close.run(());
    };

    let start_scanner = move || {
        scanner_error.set(None);

        // Small delay to let the DOM element render
        let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            let elem_id_inner = element_id.get_value();
            match qr_scanner::start_qr_scanner(&elem_id_inner, move |decoded_text| {
                let input = decoded_text.trim().to_string();
                match parse_oob_notes(&input) {
                    Ok(parsed) => {
                        // Deduplicate at paper note level
                        let paper_note_id = parsed.notes.first()
                            .map(|n| n.paper_note_id)
                            .unwrap_or_default();

                        let already_scanned = scanned_paper_ids.get_untracked().contains(&paper_note_id);
                        let already_in_set = state
                            .get_note_set(set_id)
                            .map(|s| s.has_paper_note(paper_note_id))
                            .unwrap_or(false);

                        if already_scanned || already_in_set {
                            state.add_toast(
                                "Notes already imported".to_string(),
                                ToastVariant::Info,
                            );
                        } else {
                            scanned_paper_ids.update(|s| {
                                s.insert(paper_note_id);
                            });
                            match state.add_notes_to_set(set_id, parsed.notes, parsed.federation_id) {
                                Ok(count) => {
                                    if count > 0 {
                                        imported_notes.set(true);
                                    }
                                    state.add_toast(
                                        format!("Added {} paper note(s)", count),
                                        ToastVariant::Success,
                                    );
                                }
                                Err(e) => {
                                    state.add_toast(
                                        format!("Rejected: {}", e),
                                        ToastVariant::Error,
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        state.add_toast(format!("Invalid QR: {}", e), ToastVariant::Warning);
                    }
                }
            }) {
                Ok(()) => {
                    scanner_active.set(true);
                }
                Err(e) => {
                    scanner_error.set(Some(e));
                }
            }
        }) as Box<dyn FnMut()>);

        let _ = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                200,
            );
        cb.forget();
    };

    // Auto-start scanner when modal opens
    Effect::new(move || {
        if open.get() {
            start_scanner();
        }
    });

    view! {
        <Modal open=open on_close=Callback::new(move |()| handle_close()) title="Scan QR Code">
            {move || scanner_error.get().map(|msg| view! {
                <div class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-lg text-sm">
                    {msg}
                </div>
            })}

            <div
                id=element_id.get_value()
                class="w-full mb-4 min-h-[300px] bg-gray-100 dark:bg-gray-700 rounded-lg overflow-hidden"
            ></div>

            <p class="text-sm text-gray-500 dark:text-gray-400 text-center mb-4">
                "Point your camera at an ecash QR code. Notes are added automatically."
            </p>

            <div class="flex justify-end">
                <Button
                    variant=ButtonVariant::Outline
                    on_click=Callback::new(move |_| handle_close())
                >
                    "Close"
                </Button>
            </div>
        </Modal>
    }
}
