use leptos::prelude::*;

#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-12 px-4 text-center">
            <div class="w-16 h-16 mb-4 text-gray-400 dark:text-gray-600">
                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" class="w-full h-full">
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="1.5"
                        d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
                    />
                </svg>
            </div>
            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-1">
                {title}
            </h3>
            {if !description.is_empty() {
                view! {
                    <p class="text-sm text-gray-500 dark:text-gray-400 mb-4 max-w-sm">
                        {description}
                    </p>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            {children.map(|c| c())}
        </div>
    }
}
