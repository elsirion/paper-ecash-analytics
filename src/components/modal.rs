use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into, optional)] title: String,
    children: ChildrenFn,
) -> impl IntoView {
    let handle_backdrop_click = move |_: web_sys::MouseEvent| {
        on_close.run(());
    };

    let handle_content_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
    };

    view! {
        <Show when=move || open.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                on:click=handle_backdrop_click
            >
                <div
                    class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full mx-4 max-h-[90vh] overflow-auto"
                    on:click=handle_content_click
                >
                    <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                            {title.clone()}
                        </h3>
                        <button
                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                            on:click=move |_| on_close.run(())
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                    <div class="p-4">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
