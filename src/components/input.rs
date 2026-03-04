use leptos::prelude::*;

#[allow(dead_code)]
#[component]
pub fn Input(
    #[prop(into, optional)] value: RwSignal<String>,
    #[prop(into, optional)] placeholder: String,
    #[prop(into, optional)] label: String,
    #[prop(into, optional)] input_type: String,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let input_type = if input_type.is_empty() {
        "text".to_string()
    } else {
        input_type
    };

    view! {
        <div class=format!("w-full {}", class)>
            {if !label.is_empty() {
                view! {
                    <label class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                        {label.clone()}
                    </label>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <input
                type=input_type
                class="input"
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
                disabled=move || disabled.get()
            />
        </div>
    }
}

#[allow(dead_code)]
#[component]
pub fn TextArea(
    #[prop(into, optional)] value: RwSignal<String>,
    #[prop(into, optional)] placeholder: String,
    #[prop(into, optional)] label: String,
    #[prop(into, optional)] rows: u32,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let rows = if rows == 0 { 4 } else { rows };

    view! {
        <div class=format!("w-full {}", class)>
            {if !label.is_empty() {
                view! {
                    <label class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                        {label.clone()}
                    </label>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <textarea
                class="input resize-none"
                placeholder=placeholder
                rows=rows
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
                disabled=move || disabled.get()
            ></textarea>
        </div>
    }
}
