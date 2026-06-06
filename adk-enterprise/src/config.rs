//! ClientConfig — configuration for the EnterpriseClient.

use std::time::Duration;

/// Default production base URL for the managed agent service.
pub const DEFAULT_BASE_URL: &str = "https://enterprise.adk-rust.com/managed/v1";

/// Default API version header value.
pub const DEFAULT_VERSION: &str = "2026-06-01";

/// Configuration for the EnterpriseClient.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API key (`adk_live_…` or `adk_test_…`).
    pub api_key: String,
    /// Base URL for the managed agent service.
    pub base_url: String,
    /// Required version header value.
    pub version: String,
    /// SSE stream timeout (how long to wait for data before reconnecting).
    pub sse_timeout: Duration,
    /// Maximum retry attempts for transient errors.
    pub max_retries: u32,
    /// Initial backoff duration for retries.
    pub retry_backoff: Duration,
}

impl ClientConfig {
    /// Create a new configuration with the given API key and production defaults.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.into(),
            version: DEFAULT_VERSION.into(),
            sse_timeout: Duration::from_secs(300),
            max_retries: 3,
            retry_backoff: Duration::from_secs(1),
        }
    }

    /// Create a configuration for a self-hosted deployment.
    pub fn self_hosted(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            version: DEFAULT_VERSION.into(),
            sse_timeout: Duration::from_secs(300),
            max_retries: 3,
            retry_backoff: Duration::from_secs(1),
        }
    }

    /// Set the base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the SSE stream timeout.
    pub fn with_sse_timeout(mut self, timeout: Duration) -> Self {
        self.sse_timeout = timeout;
        self
    }

    /// Set the maximum retry count.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the initial retry backoff duration.
    pub fn with_retry_backoff(mut self, backoff: Duration) -> Self {
        self.retry_backoff = backoff;
        self
    }
}
