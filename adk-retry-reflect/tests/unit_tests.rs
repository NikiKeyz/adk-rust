//! Unit tests for the Retry & Reflect plugin.
//!
//! Tests builder defaults, error cases, reset behavior, and configuration validation.

use std::collections::HashSet;
use std::time::Duration;

use adk_retry_reflect::{
    BackoffStrategy, RetryReflectError, RetryReflectPluginBuilder, ToolFilter,
};

#[test]
fn test_builder_defaults() {
    let plugin = RetryReflectPluginBuilder::new().build().expect("default config should be valid");

    let config = plugin.config();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.priority, 200);
    assert_eq!(config.backoff, BackoffStrategy::None);
    assert_eq!(config.tool_filter, ToolFilter::None);
    assert_eq!(config.max_backoff, Duration::from_secs(30));
    assert_eq!(config.global_limit, None);
    assert!(!config.global_tracking);
    assert_eq!(config.global_failure_threshold, 10);
    assert!(config.per_tool_limits.is_empty());
}

#[test]
fn test_default_template_contains_all_placeholders() {
    let plugin = RetryReflectPluginBuilder::new().build().expect("default config should be valid");

    let template = &plugin.config().template;
    assert!(template.contains("{tool_name}"));
    assert!(template.contains("{args}"));
    assert!(template.contains("{error}"));
    assert!(template.contains("{attempt}"));
    assert!(template.contains("{max_retries}"));
    assert!(template.contains("{guidance}"));
}

#[test]
fn test_max_retries_zero_rejected() {
    let result = RetryReflectPluginBuilder::new().max_retries(0).build();

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        RetryReflectError::InvalidMaxRetries { value } => assert_eq!(value, 0),
        _ => panic!("expected InvalidMaxRetries, got {err:?}"),
    }
}

#[test]
fn test_custom_max_retries() {
    let plugin = RetryReflectPluginBuilder::new().max_retries(7).build().expect("valid config");

    assert_eq!(plugin.config().max_retries, 7);
}

#[test]
fn test_per_tool_limit() {
    let plugin = RetryReflectPluginBuilder::new()
        .per_tool_limit("search", 10)
        .per_tool_limit("delete", 1)
        .build()
        .expect("valid config");

    assert_eq!(plugin.config().per_tool_limits.get("search"), Some(&10));
    assert_eq!(plugin.config().per_tool_limits.get("delete"), Some(&1));
}

#[test]
fn test_global_limit() {
    let plugin = RetryReflectPluginBuilder::new().global_limit(20).build().expect("valid config");

    assert_eq!(plugin.config().global_limit, Some(20));
}

#[test]
fn test_backoff_fixed() {
    let plugin = RetryReflectPluginBuilder::new()
        .backoff_fixed(Duration::from_secs(2))
        .build()
        .expect("valid config");

    assert_eq!(plugin.config().backoff, BackoffStrategy::Fixed(Duration::from_secs(2)));
}

#[test]
fn test_backoff_exponential() {
    let plugin = RetryReflectPluginBuilder::new()
        .backoff_exponential(Duration::from_millis(100))
        .build()
        .expect("valid config");

    assert_eq!(
        plugin.config().backoff,
        BackoffStrategy::Exponential { base_delay: Duration::from_millis(100) }
    );
}

#[test]
fn test_max_backoff_ceiling() {
    let plugin = RetryReflectPluginBuilder::new()
        .max_backoff(Duration::from_secs(60))
        .build()
        .expect("valid config");

    assert_eq!(plugin.config().max_backoff, Duration::from_secs(60));
}

#[test]
fn test_allowlist_filter() {
    let plugin = RetryReflectPluginBuilder::new()
        .allowlist(["search", "fetch"])
        .build()
        .expect("valid config");

    let expected: HashSet<String> = HashSet::from(["search".to_string(), "fetch".to_string()]);
    assert_eq!(plugin.config().tool_filter, ToolFilter::Allowlist(expected));
}

#[test]
fn test_denylist_filter() {
    let plugin = RetryReflectPluginBuilder::new()
        .denylist(["delete", "drop"])
        .build()
        .expect("valid config");

    let expected: HashSet<String> = HashSet::from(["delete".to_string(), "drop".to_string()]);
    assert_eq!(plugin.config().tool_filter, ToolFilter::Denylist(expected));
}

#[test]
fn test_custom_template() {
    let custom = "Error in {tool_name}: {error}";
    let plugin = RetryReflectPluginBuilder::new().template(custom).build().expect("valid config");

    assert_eq!(plugin.config().template, custom);
}

#[test]
fn test_custom_priority() {
    let plugin = RetryReflectPluginBuilder::new().priority(50).build().expect("valid config");

    assert_eq!(plugin.config().priority, 50);
}

#[test]
fn test_global_tracking_enabled() {
    let plugin =
        RetryReflectPluginBuilder::new().enable_global_tracking(15).build().expect("valid config");

    assert!(plugin.config().global_tracking);
    assert_eq!(plugin.config().global_failure_threshold, 15);
}

#[tokio::test]
async fn test_reset_clears_per_invocation_state() {
    let plugin = RetryReflectPluginBuilder::new().build().expect("valid config");

    // Reset should not panic on empty state
    plugin.reset().await;
}

#[test]
fn test_global_tracking_disabled_by_default() {
    let plugin = RetryReflectPluginBuilder::new().build().expect("valid config");

    assert!(!plugin.config().global_tracking);
}
