use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
}

impl ButtonVariant {
    fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "btn btn-primary",
            ButtonVariant::Secondary => "btn btn-secondary",
            ButtonVariant::Outline => "btn btn-outline",
            ButtonVariant::Ghost => "btn btn-ghost",
            ButtonVariant::Danger => "btn btn-danger",
        }
    }
}

#[component]
pub fn Button(
    #[prop(into, optional)] variant: ButtonVariant,
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] disabled: MaybeSignal<bool>,
    #[prop(into, optional)] loading: MaybeSignal<bool>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let base_classes = variant.classes();

    let handle_click = move |_| {
        if let Some(cb) = on_click {
            cb.run(());
        }
    };

    view! {
        <button
            class=move || {
                format!(
                    "{} {} {}",
                    base_classes,
                    class,
                    if disabled.get() || loading.get() { "opacity-50 cursor-not-allowed" } else { "" }
                )
            }
            disabled=move || disabled.get() || loading.get()
            on:click=handle_click
        >
            {move || {
                if loading.get() {
                    view! {
                        <svg
                            class="animate-spin -ml-1 mr-2 h-4 w-4"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                        >
                            <circle
                                class="opacity-25"
                                cx="12"
                                cy="12"
                                r="10"
                                stroke="currentColor"
                                stroke-width="4"
                            ></circle>
                            <path
                                class="opacity-75"
                                fill="currentColor"
                                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            ></path>
                        </svg>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
            {children()}
        </button>
    }
}
