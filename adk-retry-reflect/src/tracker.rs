//! Per-invocation and global retry state tracking.

use std::collections::HashMap;

/// Per-invocation state tracking for retry counts.
///
/// Maintains failure counts keyed by `"{tool_name}:{call_id}"` composite key.
/// Thread safety is provided by wrapping in `Arc<Mutex<...>>` at the plugin level.
#[derive(Debug, Default)]
pub struct RetryTracker {
    /// Failure counts keyed by "tool_name:call_id".
    counts: HashMap<String, u32>,
    /// Total retries in this invocation (for global limit).
    total_retries: u32,
}

impl RetryTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the failure count for the given key and return the new count.
    pub fn increment(&mut self, key: &str) -> u32 {
        let count = self.counts.entry(key.to_string()).or_insert(0);
        *count += 1;
        self.total_retries += 1;
        *count
    }

    /// Get the current failure count for the given key.
    pub fn get(&self, key: &str) -> u32 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    /// Get the total number of retries across all tools in this invocation.
    pub fn total(&self) -> u32 {
        self.total_retries
    }

    /// Reset all failure counts (called between agent invocations).
    pub fn reset(&mut self) {
        self.counts.clear();
        self.total_retries = 0;
    }
}

/// Cross-invocation state for circuit-breaker patterns.
///
/// Persists failure counts across multiple agent invocations to detect
/// chronically failing tools.
#[derive(Debug, Default)]
pub struct GlobalRetryTracker {
    /// Total failure counts per tool name across all invocations.
    tool_failures: HashMap<String, u32>,
}

impl GlobalRetryTracker {
    /// Create a new empty global tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a failure for the given tool and return the new total failure count.
    pub fn record_failure(&mut self, tool_name: &str) -> u32 {
        let count = self.tool_failures.entry(tool_name.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    /// Check if a tool has exceeded the circuit-breaker threshold.
    pub fn is_circuit_broken(&self, tool_name: &str, threshold: u32) -> bool {
        self.tool_failures.get(tool_name).copied().unwrap_or(0) >= threshold
    }

    /// Reset all global failure counts.
    pub fn reset(&mut self) {
        self.tool_failures.clear();
    }
}
