use leptos::prelude::*;

use crate::components::{Badge, BadgeVariant};
use crate::models::{Note, NoteStatus};
use crate::utils::encoding::{format_amount_msat, format_nonce};
use crate::utils::time::format_relative_time;

#[component]
pub fn NoteRow(note: Note) -> impl IntoView {
    let (status_badge, status_info) = match &note.status {
        NoteStatus::Unspent => (
            view! { <Badge variant=BadgeVariant::Green>"Unspent"</Badge> },
            None,
        ),
        NoteStatus::Spent(info) => (
            view! { <Badge variant=BadgeVariant::Red>"Spent"</Badge> },
            Some(format!(
                "Session #{} - {}",
                info.session_index,
                info.estimated_timestamp
                    .map(|t| format_relative_time(&t))
                    .unwrap_or_else(|| "Unknown time".to_string())
            )),
        ),
        NoteStatus::Error(msg) => (
            view! { <Badge variant=BadgeVariant::Yellow>"Error"</Badge> },
            Some(msg.clone()),
        ),
    };

    view! {
        <tr class="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800">
            <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                {note.index + 1}
            </td>
            <td class="px-4 py-3 font-mono text-sm text-gray-700 dark:text-gray-300">
                {format_nonce(&note.nonce)}
            </td>
            <td class="px-4 py-3 text-sm text-gray-900 dark:text-white text-right">
                {format_amount_msat(note.amount_msat)}
            </td>
            <td class="px-4 py-3">
                {status_badge}
            </td>
            <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                {status_info.unwrap_or_default()}
            </td>
            <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                {note.last_checked.map(|t| format_relative_time(&t)).unwrap_or_else(|| "-".to_string())}
            </td>
        </tr>
    }
}
