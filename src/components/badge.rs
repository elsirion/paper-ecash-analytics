use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Gray,
    Green,
    Red,
    Yellow,
}

impl BadgeVariant {
    fn classes(&self) -> &'static str {
        match self {
            BadgeVariant::Gray => "badge badge-gray",
            BadgeVariant::Green => "badge badge-green",
            BadgeVariant::Red => "badge badge-red",
            BadgeVariant::Yellow => "badge badge-yellow",
        }
    }
}

#[component]
pub fn Badge(
    #[prop(into, optional)] variant: BadgeVariant,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <span class=format!("{} {}", variant.classes(), class)>
            {children()}
        </span>
    }
}
