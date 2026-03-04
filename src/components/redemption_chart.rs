use std::collections::BTreeMap;

use chrono::{DateTime, Timelike, Utc};
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

/// Format timestamp for display (short)
fn format_date(dt: &DateTime<Utc>) -> String {
    dt.format("%b %d").to_string()
}

/// Format timestamp for tooltip (full date)
fn format_date_full(dt: &DateTime<Utc>) -> String {
    dt.format("%b %d, %Y").to_string()
}

/// Data point for hour-of-day chart
#[derive(Clone, Debug)]
struct HourPoint {
    hour: u32,
    count: u32,
}

/// Aggregate redemptions by hour of day (0-23)
fn aggregate_by_hour(note_set: &NoteSet) -> Vec<HourPoint> {
    let mut hours: [u32; 24] = [0; 24];

    for note in &note_set.notes {
        if let NoteStatus::Spent(spend_info) = &note.status {
            if let Some(ts) = spend_info.estimated_timestamp {
                let hour = ts.hour() as usize;
                hours[hour] += 1;
            }
        }
    }

    hours
        .iter()
        .enumerate()
        .map(|(hour, &count)| HourPoint {
            hour: hour as u32,
            count,
        })
        .collect()
}

/// Format hour for display (e.g., "2pm", "10am")
fn format_hour(hour: u32) -> String {
    match hour {
        0 => "12am".to_string(),
        1..=11 => format!("{}am", hour),
        12 => "12pm".to_string(),
        13..=23 => format!("{}pm", hour - 12),
        _ => format!("{}", hour),
    }
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

        let max_count = points.iter().map(|p| p.count).max().unwrap_or(1);
        let bar_count = points.len();
        let bar_width = (inner_width / bar_count as f64).min(40.0);
        let bar_gap = 4.0;
        let actual_bar_width = bar_width - bar_gap;

        points
            .iter()
            .enumerate()
            .map(|(i, point)| {
                let x = padding_left + (i as f64 * bar_width) + bar_gap / 2.0;
                let bar_height = if max_count > 0 {
                    (point.count as f64 / max_count as f64) * inner_height
                } else {
                    0.0
                };
                let y = padding_top + inner_height - bar_height;

                (
                    x,
                    y,
                    actual_bar_width,
                    bar_height,
                    point.count,
                    format_date(&point.timestamp),
                    format_date_full(&point.timestamp),
                )
            })
            .collect::<Vec<_>>()
    };

    let y_labels = move || {
        let points = data.get();
        let max_count = points.iter().map(|p| p.count).max().unwrap_or(0);

        if max_count == 0 {
            return vec![];
        }

        // Create y-axis labels for counts
        vec![
            (padding_top, format!("{}", max_count)),
            (padding_top + inner_height * 0.5, format!("{}", max_count / 2)),
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
                key=|(x, _, _, _, _, label, _)| format!("{}-{}", x, label)
                children=move |(x, y, width, height, count, label, full_date)| {
                    let tooltip = format!("{}: {} redemption{}", full_date, count, if count == 1 { "" } else { "s" });
                    view! {
                        <g>
                            // Bar with tooltip
                            <rect
                                x=x
                                y=y
                                width=width
                                height=height.max(1.0)
                                class="fill-primary-500 hover:fill-primary-600 transition-colors cursor-pointer"
                                rx="2"
                            >
                                <title>{tooltip}</title>
                            </rect>

                            // X-axis label (date)
                            <text
                                x=x + width / 2.0
                                y=padding_top + inner_height + 16.0
                                text-anchor="middle"
                                class="fill-gray-500 dark:fill-gray-400"
                                font-size="9"
                            >
                                {label}
                            </text>
                        </g>
                    }
                }
            />
        </svg>
    }
}

#[component]
pub fn HourlyRedemptionChart(
    #[prop(into)] note_set: Signal<Option<NoteSet>>,
) -> impl IntoView {
    let chart_data = move || {
        note_set.get()
            .map(|set| aggregate_by_hour(&set))
            .unwrap_or_default()
    };

    let has_data = move || chart_data().iter().any(|p| p.count > 0);

    view! {
        <Show when=has_data>
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4 mb-6">
                <div class="flex justify-between items-center mb-4">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
                        "Redemptions by Hour of Day"
                    </h2>
                    <div class="text-sm text-gray-500 dark:text-gray-400">
                        "UTC timezone"
                    </div>
                </div>

                <div class="h-48">
                    <HourlyChartSvg data=Signal::derive(chart_data) />
                </div>
            </div>
        </Show>
    }
}

#[component]
fn HourlyChartSvg(
    #[prop(into)] data: Signal<Vec<HourPoint>>,
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

        let max_count = points.iter().map(|p| p.count).max().unwrap_or(1);
        let bar_count = 24; // Always 24 hours
        let bar_width = inner_width / bar_count as f64;
        let bar_gap = 2.0;
        let actual_bar_width = bar_width - bar_gap;

        points
            .iter()
            .map(|point| {
                let x = padding_left + (point.hour as f64 * bar_width) + bar_gap / 2.0;
                let bar_height = if max_count > 0 {
                    (point.count as f64 / max_count as f64) * inner_height
                } else {
                    0.0
                };
                let y = padding_top + inner_height - bar_height;

                (
                    x,
                    y,
                    actual_bar_width,
                    bar_height,
                    point.count,
                    point.hour,
                )
            })
            .collect::<Vec<_>>()
    };

    let y_labels = move || {
        let points = data.get();
        let max_count = points.iter().map(|p| p.count).max().unwrap_or(0);

        if max_count == 0 {
            return vec![];
        }

        vec![
            (padding_top, format!("{}", max_count)),
            (padding_top + inner_height * 0.5, format!("{}", max_count / 2)),
            (padding_top + inner_height, "0".to_string()),
        ]
    };

    // X-axis labels - show every 3 hours
    let x_labels = move || {
        let bar_width = inner_width / 24.0;
        (0..24)
            .step_by(3)
            .map(|hour| {
                let x = padding_left + (hour as f64 * bar_width) + bar_width / 2.0;
                (x, format_hour(hour))
            })
            .collect::<Vec<_>>()
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

            // X-axis labels (every 3 hours)
            <For
                each=x_labels
                key=|(x, label)| format!("{}-{}", x, label)
                children=move |(x, label)| {
                    view! {
                        <text
                            x=x
                            y=padding_top + inner_height + 16.0
                            text-anchor="middle"
                            class="fill-gray-500 dark:fill-gray-400"
                            font-size="9"
                        >
                            {label}
                        </text>
                    }
                }
            />

            // Bars
            <For
                each=bars
                key=|(_, _, _, _, _, hour)| *hour
                children=move |(x, y, width, height, count, hour)| {
                    let tooltip = format!("{}: {} redemption{}", format_hour(hour), count, if count == 1 { "" } else { "s" });
                    view! {
                        <rect
                            x=x
                            y=y
                            width=width
                            height=height.max(1.0)
                            class="fill-emerald-500 hover:fill-emerald-600 transition-colors cursor-pointer"
                            rx="2"
                        >
                            <title>{tooltip}</title>
                        </rect>
                    }
                }
            />
        </svg>
    }
}
