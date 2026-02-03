use std::path::Path;

use anyhow::{Context, Result};

/// Get file name without extension
pub fn get_file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

/// Convert a path to a UTF-8 string with a helpful error message.
pub fn path_to_str<'a>(path: &'a Path, label: &str) -> Result<&'a str> {
    path.to_str()
        .context(format!("{label} path is not valid UTF-8: {path:?}"))
}

/// Format duration in HH:MM:SS
pub fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// Format timestamp in MM:SS.ms
pub fn format_timestamp(seconds: f64) -> String {
    let minutes = (seconds / 60.0) as u32;
    let secs = seconds % 60.0;

    format!("{:02}:{:05.2}", minutes, secs)
}
