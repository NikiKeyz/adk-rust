//! Error detection from tool result JSON values.

use serde_json::Value;

/// Determine whether a tool result represents an error.
///
/// A result is considered an error if:
/// 1. It is a JSON object with an `"error"` key at the top level, OR
/// 2. It is a JSON object with `"isError": true`, OR
/// 3. It is a JSON string starting with `"Error:"` or `"error:"`
///
/// # Example
///
/// ```rust
/// use adk_retry_reflect::detection::is_error_result;
/// use serde_json::json;
///
/// assert!(is_error_result(&json!({"error": "not found"})));
/// assert!(is_error_result(&json!({"isError": true})));
/// assert!(is_error_result(&json!("Error: connection refused")));
/// assert!(!is_error_result(&json!({"result": "ok"})));
/// assert!(!is_error_result(&json!(42)));
/// ```
pub fn is_error_result(result: &Value) -> bool {
    match result {
        Value::Object(map) => {
            map.contains_key("error")
                || map.get("isError").and_then(|v| v.as_bool()).unwrap_or(false)
        }
        Value::String(s) => s.starts_with("Error:") || s.starts_with("error:"),
        _ => false,
    }
}
