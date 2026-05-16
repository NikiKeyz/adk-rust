//! Reflection prompt template rendering.

/// Default reflection template used when no custom template is configured.
pub const DEFAULT_TEMPLATE: &str = "\
[RETRY GUIDANCE - Attempt {attempt}/{max_retries}]

Tool: {tool_name}
Error: {error}

Original Arguments:
{args}

{guidance}

Please analyze the error above and retry the tool call with corrected arguments.";

/// Render a reflection prompt by substituting placeholders in the template.
///
/// # Placeholders
///
/// - `{tool_name}` — Name of the failed tool
/// - `{args}` — JSON-serialized original arguments
/// - `{error}` — Original error message verbatim
/// - `{attempt}` — Current attempt number (1-indexed)
/// - `{max_retries}` — Maximum retries configured for this tool
/// - `{guidance}` — Optional custom guidance text
///
/// # Example
///
/// ```rust
/// use adk_retry_reflect::template::{render_reflection, DEFAULT_TEMPLATE};
///
/// let result = render_reflection(
///     DEFAULT_TEMPLATE,
///     "my_tool",
///     r#"{"key": "value"}"#,
///     "connection timeout",
///     1,
///     3,
///     "",
/// );
/// assert!(result.contains("my_tool"));
/// assert!(result.contains("connection timeout"));
/// ```
pub fn render_reflection(
    template: &str,
    tool_name: &str,
    args: &str,
    error: &str,
    attempt: u32,
    max_retries: u32,
    guidance: &str,
) -> String {
    template
        .replace("{tool_name}", tool_name)
        .replace("{args}", args)
        .replace("{error}", error)
        .replace("{attempt}", &attempt.to_string())
        .replace("{max_retries}", &max_retries.to_string())
        .replace("{guidance}", guidance)
}
