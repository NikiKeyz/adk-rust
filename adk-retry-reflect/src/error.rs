//! Error types for the Retry & Reflect plugin.

use thiserror::Error;

/// Errors that can occur during plugin configuration or operation.
#[derive(Debug, Error)]
pub enum RetryReflectError {
    /// Both an allowlist and a denylist were provided in the tool filter configuration.
    #[error("Invalid configuration: both allowlist and denylist are set")]
    ConflictingFilter,

    /// The configured max retries value is invalid (must be at least 1).
    #[error("Max retries must be at least 1, got {value}")]
    InvalidMaxRetries {
        /// The invalid value that was provided.
        value: u32,
    },
}
