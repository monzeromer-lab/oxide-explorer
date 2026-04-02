use chrono::{DateTime, Local};

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_date(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp, 0)
        .map(|utc| utc.with_timezone(&Local));
    match dt {
        Some(local) => local.format("%Y-%m-%d %H:%M").to_string(),
        None => String::from("—"),
    }
}
