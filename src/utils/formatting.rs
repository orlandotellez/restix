/// Format JSON with indentation for readability
pub fn format_json(value: &str) -> String {
    // Try to parse as JSON and pretty print
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| value.to_string())
    } else {
        value.to_string()
    }
}

/// Try to format value as JSON if it looks like JSON
pub fn try_format_as_json(value: &str) -> String {
    // Check if it looks like JSON (starts with { or [)
    let trimmed = value.trim();
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        format_json(value)
    } else {
        value.to_string()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_ttl(ttl: i64) -> String {
    if ttl == -1 {
        "No expiration".to_string()
    } else if ttl < 0 {
        "Expired".to_string()
    } else if ttl < 60 {
        format!("{}s", ttl)
    } else if ttl < 3600 {
        format!("{}m {}s", ttl / 60, ttl % 60)
    } else {
        format!("{}h {}m", ttl / 3600, (ttl % 3600) / 60)
    }
}

pub fn get_type_badge(key_type: &str) -> &'static str {
    match key_type.to_lowercase().as_str() {
        "string" => "S",
        "list" => "L",
        "hash" => "H",
        "set" => "U",
        "zset" => "Z",
        _ => "?",
    }
}
