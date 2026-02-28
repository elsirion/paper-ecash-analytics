use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use leptos::prelude::*;

use crate::models::{NoteSet, NoteStatus};

/// Data point for the chart
#[derive(Clone, Debug)]
struct ChartPoint {
    timestamp: DateTime<Utc>,
    amount_sats: u64,
    count: u32,
}

/// Aggregate redemptions by time bucket for a single note set
fn aggregate_redemptions(note_set: &NoteSet, bucket_hours: i64) -> Vec<ChartPoint> {
    let mut buckets: BTreeMap<i64, (u64, u32)> = BTreeMap::new();

    for note in &note_set.notes {
        if let NoteStatus::Spent(spend_info) = &note.status {
            if let Some(ts) = spend_info.estimated_timestamp {
                // Round down to bucket
                let bucket_ts = (ts.timestamp() / (bucket_hours * 3600)) * (bucket_hours * 3600);
                let entry = buckets.entry(bucket_ts).or_insert((0, 0));
                entry.0 += note.amount_msat / 1000; // Convert to sats
                entry.1 += 1;
            }
        }
    }

    buckets
        .into_iter()
        .map(|(ts, (amount_sats, count))| ChartPoint {
            timestamp: DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now),
            amount_sats,
            count,
        })
        .collect()
}

/// Format timestamp for display
fn format_date(dt: &DateTime<Utc>) -> String {
    dt.format("%b %d").to_string()
}

#[component]
pub fn RedemptionChart(
    #[prop(into)] note_set: Signal<Option<NoteSet>>,
) -> impl IntoView {
    let chart_data = move || {
        note_set.get()
            .map(|set| aggregate_redemptions(&set, 24))
            .unwrap_or_default()
    };

    let has_data = move || !chart_data().is_empty();

    let total_redeemed = move || {
        chart_data().iter().map(|p| p.amount_sats).sum::<u64>()
    };

    let total_count = move || {
        chart_data().iter().map(|p| p.count).sum::<u32>()
    };

    view! {
        <Show when=has_data>
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4 mb-6">
                <div class="flex justify-between items-center mb-4">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
                        "Redemptions Over Time"
                    </h2>
                    <div class="text-sm text-gray-500 dark:text-gray-400">
                        <span class="font-medium text-gray-900 dark:text-white">
                            {move || format!("{}", total_redeemed())}
                        </span>
                        " sats in "
                        <span class="font-medium text-gray-900 dark:text-white">
                            {move || format!("{}", total_count())}
                        </span>
                        " redemptions"
                    </div>
                </div>

                <div class="h-48">
                    <ChartSvg data=Signal::derive(chart_data) />
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ChartSvg(
    #[prop(into)] data: Signal<Vec<ChartPoint>>,
) -> impl IntoView {
    let chart_width = 800.0;
    let chart_height = 160.0;
    let padding_left = 50.0;
    let padding_right = 20.0;
    let padding_top = 10.0;
    let padding_bottom = 30.0;

    let inner_width = chart_width - padding_left - padding_right;
    let inner_height = chart_height - padding_top - padding_bottom;

    let bars = move || {
        let points = data.get();
        if points.is_empty() {
            return vec![];
        }

        let max_amount = points.iter().map(|p| p.amount_sats).max().unwrap_or(1);
        let bar_count = points.len();
        let bar_width = (inner_width / bar_count as f64).min(40.0);
        let bar_gap = 4.0;
        let actual_bar_width = bar_width - bar_gap;

        points
            .iter()
            .enumerate()
            .map(|(i, point)| {
                let x = padding_left + (i as f64 * bar_width) + bar_gap / 2.0;
                let bar_height = if max_amount > 0 {
                    (point.amount_sats as f64 / max_amount as f64) * inner_height
                } else {
                    0.0
                };
                let y = padding_top + inner_height - bar_height;

                (
                    x,
                    y,
                    actual_bar_width,
                    bar_height,
                    point.amount_sats,
                    point.count,
                    format_date(&point.timestamp),
                )
            })
            .collect::<Vec<_>>()
    };

    let y_labels = move || {
        let points = data.get();
        let max_amount = points.iter().map(|p| p.amount_sats).max().unwrap_or(0);

        if max_amount == 0 {
            return vec![];
        }

        // Create 4 y-axis labels
        vec![
            (padding_top, format!("{}", max_amount)),
            (padding_top + inner_height * 0.33, format!("{}", max_amount * 2 / 3)),
            (padding_top + inner_height * 0.66, format!("{}", max_amount / 3)),
            (padding_top + inner_height, "0".to_string()),
        ]
    };

    view! {
        <svg
            viewBox=format!("0 0 {} {}", chart_width, chart_height)
            class="w-full h-full"
            preserveAspectRatio="xMidYMid meet"
        >
            // Y-axis line
            <line
                x1=padding_left
                y1=padding_top
                x2=padding_left
                y2=padding_top + inner_height
                stroke="currentColor"
                stroke-opacity="0.2"
                stroke-width="1"
            />

            // X-axis line
            <line
                x1=padding_left
                y1=padding_top + inner_height
                x2=padding_left + inner_width
                y2=padding_top + inner_height
                stroke="currentColor"
                stroke-opacity="0.2"
                stroke-width="1"
            />

            // Y-axis labels
            <For
                each=y_labels
                key=|(y, label)| format!("{}-{}", y, label)
                children=move |(y, label)| {
                    view! {
                        <text
                            x=padding_left - 8.0
                            y=y + 4.0
                            text-anchor="end"
                            class="fill-gray-500 dark:fill-gray-400"
                            font-size="10"
                        >
                            {label}
                        </text>
                    }
                }
            />

            // Bars
            <For
                each=bars
                key=|(x, _, _, _, _, _, label)| format!("{}-{}", x, label)
                children=move |(x, y, width, height, amount, count, label)| {
                    view! {
                        <g class="group">
                            // Bar
                            <rect
                                x=x
                                y=y
                                width=width
                                height=height.max(1.0)
                                class="fill-primary-500 hover:fill-primary-600 transition-colors"
                                rx="2"
                            />

                            // X-axis label (date) - show for every bar or every few bars
                            <text
                                x=x + width / 2.0
                                y=padding_top + inner_height + 16.0
                                text-anchor="middle"
                                class="fill-gray-500 dark:fill-gray-400"
                                font-size="9"
                            >
                                {label}
                            </text>

                            // Tooltip on hover
                            <title>
                                {format!("{} sats ({} notes)", amount, count)}
                            </title>
                        </g>
                    }
                }
            />
        </svg>
    }
}
