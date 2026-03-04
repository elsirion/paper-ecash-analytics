use chrono::{DateTime, Utc};

/// Format a timestamp for display
#[allow(dead_code)]
pub fn format_timestamp(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format a timestamp as relative time (e.g., "2 hours ago")
pub fn format_relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_seconds() < 0 {
        return "in the future".to_string();
    }

    if duration.num_seconds() < 60 {
        return "just now".to_string();
    }

    if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        return format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" });
    }

    if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
    }

    if duration.num_days() < 30 {
        let days = duration.num_days();
        return format!("{} day{} ago", days, if days == 1 { "" } else { "s" });
    }

    if duration.num_days() < 365 {
        let months = duration.num_days() / 30;
        return format!("{} month{} ago", months, if months == 1 { "" } else { "s" });
    }

    let years = duration.num_days() / 365;
    format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
}

/// Format a duration in seconds as human-readable
#[allow(dead_code)]
pub fn format_duration_secs(secs: u64) -> String {
    if secs < 60 {
        return format!("{}s", secs);
    }

    if secs < 3600 {
        let mins = secs / 60;
        return format!("{}m", mins);
    }

    if secs < 86400 {
        let hours = secs / 3600;
        return format!("{}h", hours);
    }

    let days = secs / 86400;
    format!("{}d", days)
}
