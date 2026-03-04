use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::{Button, ButtonVariant, Modal};
use crate::models::NoteSet;
use crate::state::{use_app_state, ToastVariant};

#[component]
pub fn CreateNoteSetModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let state = use_app_state();

    let name_input = RwSignal::new(String::new());
    let error_message = RwSignal::new(Option::<String>::None);
    let navigate_to = RwSignal::new(Option::<String>::None);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let schedule_close = move || {
        name_input.set(String::new());
        error_message.set(None);
        on_close.run(());
    };

    // Watch for navigation requests
    Effect::new(move || {
        if let Some(path) = navigate_to.get() {
            navigate_to.set(None);
            let navigate = use_navigate();
            navigate(&path, Default::default());
        }
    });

    // Focus the input when modal opens
    Effect::new(move || {
        if open.get() {
            request_animation_frame(move || {
                if let Some(el) = input_ref.get() {
                    let _ = el.focus();
                }
            });
        }
    });

    let do_create = move || {
        let name = name_input.get();
        if name.trim().is_empty() {
            error_message.set(Some("Please enter a name".to_string()));
            return;
        }

        let note_set = NoteSet::new_empty(name.trim().to_string());
        let set_id = note_set.id;
        state.add_note_set(note_set);
        state.add_toast("Note set created".to_string(), ToastVariant::Success);
        schedule_close();
        navigate_to.set(Some(format!("/set/{}", set_id)));
    };

    view! {
        <Modal open=open on_close=Callback::new(move |()| schedule_close()) title="New Note Set">
            // Error message
            {move || error_message.get().map(|msg| view! {
                <div class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-lg text-sm">
                    {msg}
                </div>
            })}

            // Name input
            <div class="mb-4">
                <label class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                    "Name"
                </label>
                <input
                    type="text"
                    class="input"
                    placeholder="My Notes"
                    node_ref=input_ref
                    prop:value=move || name_input.get()
                    on:input=move |ev| name_input.set(event_target_value(&ev))
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            do_create();
                        }
                    }
                />
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
                    on_click=Callback::new(move |_| do_create())
                >
                    "Create"
                </Button>
            </div>
        </Modal>
    }
}
