use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("card {}", class)>
            {children()}
        </div>
    }
}
