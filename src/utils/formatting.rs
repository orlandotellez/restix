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
