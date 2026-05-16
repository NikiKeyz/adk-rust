//! YAML Agent Definition loading, interpolation, serialization, and hot reload.
//!
//! This module provides a complete YAML-based agent configuration layer:
//!
//! - [`schema`] — YAML schema types for agent definitions, including model config,
//!   tool references, sub-agent references, plugin references, and session/memory config
//! - [`loader`] — Agent config loader for parsing, interpolating, validating, and
//!   resolving YAML files into live `Agent` instances
//! - [`interpolator`] — Environment variable interpolation with `${VAR}` and
//!   `${VAR:-default}` syntax for all string fields
//! - [`serializer`] — Round-trip YAML serialization of agent definitions
//! - [`watcher`] — Hot reload watcher for filesystem changes with debouncing
//!
//! ## Environment Variable Interpolation
//!
//! All string fields in a YAML agent definition support environment variable
//! placeholders:
//!
//! - `${VARIABLE_NAME}` — replaced with the env var value; error if unset
//! - `${VARIABLE_NAME:-default_value}` — uses the default if the var is unset
//!
//! Interpolation is applied automatically during `load_file()` before validation.
//!
//! ## Plugins, Session, and Memory
//!
//! Agent definitions can reference plugins by name and configure session/memory
//! backends declaratively:
//!
//! ```yaml
//! plugins:
//!   - name: telemetry
//!     config:
//!       endpoint: "${OTEL_ENDPOINT:-http://localhost:4317}"
//!
//! session:
//!   backend: postgres
//!   connection_string: "${DATABASE_URL}"
//!
//! memory:
//!   backend: inmemory
//! ```
//!
//! Enabled by the `yaml-agent` feature flag.

pub mod interpolator;
pub mod loader;
pub mod schema;
pub mod serializer;
pub mod watcher;

// Re-export key types for convenient access.
pub use loader::{AgentConfigLoader, ModelFactory};
pub use schema::{
    McpToolReference, MemoryConfig, ModelConfig, PluginReference, SessionConfig, SubAgentReference,
    ToolReference, YamlAgentDefinition,
};
pub use serializer::serialize_definition;
pub use watcher::HotReloadWatcher;
