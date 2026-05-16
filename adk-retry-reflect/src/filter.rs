//! Tool eligibility filtering.

use crate::config::ToolFilter;

/// Determine whether a tool is eligible for retry behavior based on the filter.
///
/// # Rules
///
/// - `ToolFilter::None` → all tools are eligible (returns `true`)
/// - `ToolFilter::Allowlist(set)` → only tools in the set are eligible
/// - `ToolFilter::Denylist(set)` → all tools except those in the set are eligible
///
/// # Example
///
/// ```rust
/// use std::collections::HashSet;
/// use adk_retry_reflect::filter::is_tool_eligible;
/// use adk_retry_reflect::config::ToolFilter;
///
/// assert!(is_tool_eligible(&ToolFilter::None, "any_tool"));
///
/// let allowlist = ToolFilter::Allowlist(HashSet::from(["search".to_string()]));
/// assert!(is_tool_eligible(&allowlist, "search"));
/// assert!(!is_tool_eligible(&allowlist, "delete"));
///
/// let denylist = ToolFilter::Denylist(HashSet::from(["delete".to_string()]));
/// assert!(is_tool_eligible(&denylist, "search"));
/// assert!(!is_tool_eligible(&denylist, "delete"));
/// ```
pub fn is_tool_eligible(filter: &ToolFilter, tool_name: &str) -> bool {
    match filter {
        ToolFilter::None => true,
        ToolFilter::Allowlist(set) => set.contains(tool_name),
        ToolFilter::Denylist(set) => !set.contains(tool_name),
    }
}
