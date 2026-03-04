use leptos::prelude::*;

use crate::components::{Badge, BadgeVariant};
use crate::models::{PaperNote, PaperNoteStatus};
use crate::utils::encoding::{format_amount_msat, format_nonce};
use crate::utils::time::format_relative_time;

#[component]
pub fn PaperNoteRow(paper_note: PaperNote) -> impl IntoView {
    let status = paper_note.status();
    let (status_badge, status_info) = match &status {
        PaperNoteStatus::Unspent => (
            view! { <Badge variant=BadgeVariant::Green>"Unspent"</Badge> },
            None,
        ),
        PaperNoteStatus::Spent => {
            let info_str = paper_note.spend_info().map(|info| {
                format!(
                    "Session #{} - {}",
                    info.session_index,
                    info.estimated_timestamp
                        .map(|t| format_relative_time(&t))
                        .unwrap_or_else(|| "Unknown time".to_string())
                )
            });
            (
                view! { <Badge variant=BadgeVariant::Red>"Spent"</Badge> },
                info_str,
            )
        }
        PaperNoteStatus::PartiallySpent => (
            view! { <Badge variant=BadgeVariant::Yellow>"Partial"</Badge> },
            Some("Some denominations spent".to_string()),
        ),
        PaperNoteStatus::Error => (
            view! { <Badge variant=BadgeVariant::Yellow>"Error"</Badge> },
            Some("Check error on one or more notes".to_string()),
        ),
    };

    let total_amount = paper_note.total_amount_msat();
    let nonce_display = format_nonce(paper_note.display_nonce());
    let last_checked = paper_note
        .last_checked()
        .map(|t| format_relative_time(&t))
        .unwrap_or_else(|| "-".to_string());
    let index_display = paper_note.index + 1;

    // Build denomination breakdown string for multi-note paper notes
    let denom_info = if paper_note.ecash_note_count() > 1 {
        let breakdown = paper_note.denomination_breakdown();
        let parts: Vec<String> = breakdown
            .iter()
            .map(|(amt, count)| {
                let sats = amt / 1000;
                if *count > 1 {
                    format!("{}x{}", count, sats)
                } else {
                    format!("{}", sats)
                }
            })
            .collect();
        Some(format!("{} notes: {}", paper_note.ecash_note_count(), parts.join(" + ")))
    } else {
        None
    };

    view! {
        <tr class="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800">
            <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                {index_display}
            </td>
            <td class="px-4 py-3 font-mono text-sm text-gray-700 dark:text-gray-300">
                <div>{nonce_display}</div>
                {denom_info.map(|info| view! {
                    <div class="text-xs text-gray-400 dark:text-gray-500 mt-0.5">{info}</div>
                })}
            </td>
            <td class="px-4 py-3 text-sm text-gray-900 dark:text-white text-right">
                {format_amount_msat(total_amount)}
            </td>
            <td class="px-4 py-3">
                {status_badge}
            </td>
            <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                {status_info.unwrap_or_default()}
            </td>
            <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
                {last_checked}
            </td>
        </tr>
    }
}
