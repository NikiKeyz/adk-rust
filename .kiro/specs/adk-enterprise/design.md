# Design Document: adk-enterprise — Enterprise Client SDK

## Overview

`adk-enterprise` is the native Rust client SDK for the ADK-Rust Enterprise Managed Agent Service. It is the developer-facing counterpart to `adk-managed` (the server-side runtime engine). Developers use this crate to create agents, start sessions, send messages, stream responses, and manage the full lifecycle of managed agents — identical in UX to `adk-anthropic::managed_agents` but targeting our own platform.

### Design Goals

1. **Familiar UX**: Developers who've used Anthropic's `ManagedAgentsClient` or OpenAI's Assistants API should feel immediately at home
2. **Lightweight**: Zero dependency on `adk-model`, `adk-runner`, `adk-agent`, or any heavy crate — only HTTP + SSE
3. **Wire-compatible**: Types serialize/deserialize to the same CANON §3.4 wire shapes as `adk-managed`
4. **Ergonomic**: Convenience methods, builder patterns, typed events, minimal boilerplate
5. **Resilient**: Auto-reconnect SSE, retry on transient errors, configurable timeouts
6. **Cross-platform**: Usable from servers, CLIs, WASM (with appropriate HTTP backend)

---

## Architecture: How It Compares

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                    CLIENT SDKs (talk to platform via HTTP/SSE)                │
│                                                                              │
│  ┌────────────────────────┐  ┌─────────────────────┐  ┌──────────────────┐  │
│  │ adk-enterprise (Rust)  │  │ adk-enterprise-py   │  │ adk-enterprise-ts│  │
│  │ THIS CRATE             │  │ (Python, from OAPI) │  │ (TS, from OAPI)  │  │
│  └───────────┬────────────┘  └──────────┬──────────┘  └────────┬─────────┘  │
│              │                           │                      │            │
│              └───────────────────────────┼──────────────────────┘            │
│                                          │ HTTPS + SSE                       │
└──────────────────────────────────────────┼───────────────────────────────────┘
                                           ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  Platform: https://enterprise.adk-rust.com/managed/v1                        │
│  (Auth, routing, billing, multi-tenancy)                                     │
└──────────────────────────────────────────────────────────────────────────────┘
                                           │
                                           ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│  adk-managed: ManagedAgentRuntime (the execution engine)                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Comparison with Existing Client Crates

| Aspect | `adk-anthropic::managed_agents` | `adk-gemini::interactions` | **`adk-enterprise`** |
|--------|--------------------------------|---------------------------|---------------------|
| **Target** | Anthropic's hosted service | Google's Interactions API | Our hosted service |
| **Models** | Claude only | Gemini only | **Any model** (via platform) |
| **Self-hostable** | No | No | **Yes** (point at any base_url) |
| **Event types** | Anthropic-specific | Gemini-specific | **CANON §3.4** (provider-neutral) |
| **SSE format** | `event:` + `data:` | Custom framing | `event:` + `data:` + `id:` (seq) |
| **Auth** | `x-api-key` header | OAuth2/ADC | `Authorization: Bearer adk_live_…` |
| **Version header** | `anthropic-version` | None | `ADK-Managed-Agent: 2026-06-01` |
| **Reconnect** | Manual | No | **Auto** (via `Last-Event-ID`) |
| **Pagination** | `{data: [...]}` | Custom | `{data, next_cursor, has_more}` |
| **Dependencies** | reqwest, serde, futures | reqwest, serde | reqwest, serde, futures |
| **Lines of code** | ~1300 (client.rs) | ~800 | ~1500 (estimated) |

### Key Insight

`adk-enterprise` follows the **exact same pattern** as `adk-anthropic::managed_agents`:
- A client struct with methods for each API endpoint
- SSE streaming via `futures::Stream<Item = Result<SessionEvent>>`
- Typed request/response structs for each operation
- Convenience methods for common patterns (send message, allow/deny tool)

The difference: it talks to OUR platform instead of Anthropic's, supports any model, and adds features they don't have (skills, self-hosted environments, provider choice).

---

## Module Structure

```
adk-enterprise/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                  # Exports, crate docs
│   ├── client.rs               # EnterpriseClient struct + all methods
│   ├── config.rs               # ClientConfig, builder
│   ├── types/
│   │   ├── mod.rs              # Re-exports
│   │   ├── agent.rs            # Agent, CreateAgentParams, UpdateAgentParams
│   │   ├── environment.rs      # Environment, CreateEnvironmentParams
│   │   ├── session.rs          # Session, CreateSessionParams, SessionStatus, Usage
│   │   ├── events.rs           # UserEvent, SessionEvent, ContentBlock, StopReason
│   │   ├── tools.rs            # ToolConfig, McpServerConfig, SkillRef, PermissionPolicy
│   │   ├── model_ref.rs        # ModelRef, Provider, ModelConfig
│   │   ├── vault.rs            # Vault, Credential (beta)
│   │   ├── memory.rs           # MemoryStore, Memory, MemoryVersion (beta)
│   │   └── pagination.rs       # ListResponse<T>, cursor helpers
│   ├── stream.rs               # SSE processing + auto-reconnect
│   ├── error.rs                # EnterpriseError enum
│   └── retry.rs                # Retry policy + exponential backoff
└── tests/
    ├── client_tests.rs
    ├── event_serialization_tests.rs
    ├── stream_tests.rs
    └── integration_tests.rs    # #[ignore] — needs real server
```

---

## Components and Interfaces

### EnterpriseClient

```rust
/// Client for the ADK-Rust Enterprise Managed Agent Service.
///
/// This is the primary entry point for all API operations. It handles
/// authentication, request formatting, response parsing, SSE streaming,
/// and automatic retry with exponential backoff.
///
/// # Example
///
/// ```rust,ignore
/// use adk_enterprise::{EnterpriseClient, CreateAgentParams, UserEvent};
/// use futures::StreamExt;
///
/// let client = EnterpriseClient::new("adk_live_...")?;
///
/// // Create an agent (any model)
/// let agent = client.create_agent(CreateAgentParams {
///     name: "My Agent".into(),
///     model: "gemini-2.5-flash".into(),  // or "gpt-4.1", "claude-sonnet-4-6", etc.
///     system: Some("You are a helpful assistant.".into()),
///     ..Default::default()
/// }).await?;
///
/// // Start a session
/// let session = client.create_session(agent.id, None).await?;
///
/// // Open stream FIRST (required ordering)
/// let mut stream = client.stream_events(&session.id).await?;
///
/// // Send a message
/// client.send_message(&session.id, "Hello!").await?;
///
/// // Process events
/// while let Some(event) = stream.next().await {
///     match event? {
///         SessionEvent::Message { content, .. } => println!("{content:?}"),
///         SessionEvent::StatusIdle { .. } => break,
///         _ => {}
///     }
/// }
/// ```
#[derive(Clone)]
pub struct EnterpriseClient {
    http: reqwest::Client,
    config: ClientConfig,
}
```

### ClientConfig

```rust
/// Configuration for the EnterpriseClient.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API key (`adk_live_…` or `adk_test_…`).
    pub api_key: String,
    /// Base URL for the managed agent service.
    /// Default: `https://enterprise.adk-rust.com/managed/v1`
    pub base_url: String,
    /// Required version header value.
    /// Default: `2026-06-01`
    pub version: String,
    /// SSE stream timeout (how long to wait for data before reconnecting).
    /// Default: 300 seconds.
    pub sse_timeout: Duration,
    /// Maximum retry attempts for transient errors.
    /// Default: 3.
    pub max_retries: u32,
    /// Initial backoff duration for retries.
    /// Default: 1 second.
    pub retry_backoff: Duration,
}

impl ClientConfig {
    pub fn new(api_key: impl Into<String>) -> Self;
    pub fn with_base_url(self, url: impl Into<String>) -> Self;
    pub fn with_sse_timeout(self, timeout: Duration) -> Self;
    pub fn self_hosted(api_key: impl Into<String>, base_url: impl Into<String>) -> Self;
}
```

### Full Method Surface

```rust
impl EnterpriseClient {
    // ─── Construction ────────────────────────────────────────────
    pub fn new(api_key: impl Into<String>) -> Result<Self>;
    pub fn from_env() -> Result<Self>;  // ADK_API_KEY or ADK_ENTERPRISE_KEY
    pub fn with_config(config: ClientConfig) -> Result<Self>;
    pub fn self_hosted(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self>;

    // ─── Agent CRUD ──────────────────────────────────────────────
    pub async fn create_agent(&self, params: CreateAgentParams) -> Result<Agent>;
    pub async fn get_agent(&self, agent_id: &str) -> Result<Agent>;
    pub async fn list_agents(&self, params: Option<ListParams>) -> Result<ListResponse<Agent>>;
    pub async fn update_agent(&self, agent_id: &str, params: UpdateAgentParams) -> Result<Agent>;
    pub async fn archive_agent(&self, agent_id: &str) -> Result<Agent>;
    pub async fn delete_agent(&self, agent_id: &str) -> Result<()>;

    // ─── Environment CRUD ────────────────────────────────────────
    pub async fn create_environment(&self, params: CreateEnvironmentParams) -> Result<Environment>;
    pub async fn get_environment(&self, env_id: &str) -> Result<Environment>;
    pub async fn archive_environment(&self, env_id: &str) -> Result<Environment>;
    pub async fn delete_environment(&self, env_id: &str) -> Result<()>;
    pub async fn download_environment(&self, env_id: &str) -> Result<Vec<u8>>; // tar snapshot

    // ─── Session CRUD ────────────────────────────────────────────
    pub async fn create_session(&self, agent_id: &str, env_id: Option<&str>) -> Result<Session>;
    pub async fn create_session_full(&self, params: CreateSessionParams) -> Result<Session>;
    pub async fn get_session(&self, session_id: &str) -> Result<Session>;
    pub async fn list_sessions(&self, params: Option<ListParams>) -> Result<ListResponse<Session>>;
    pub async fn pause_session(&self, session_id: &str) -> Result<Session>;
    pub async fn resume_session(&self, session_id: &str) -> Result<Session>;
    pub async fn archive_session(&self, session_id: &str) -> Result<Session>;
    pub async fn delete_session(&self, session_id: &str) -> Result<()>;

    // ─── Events ──────────────────────────────────────────────────
    pub async fn send_event(&self, session_id: &str, event: UserEvent) -> Result<()>;
    pub async fn stream_events(&self, session_id: &str) -> Result<EventStream>;
    pub async fn list_events(&self, session_id: &str, params: Option<ListParams>) -> Result<ListResponse<StoredEvent>>;

    // ─── Convenience Methods ─────────────────────────────────────
    pub async fn send_message(&self, session_id: &str, text: impl Into<String>) -> Result<()>;
    pub async fn interrupt(&self, session_id: &str) -> Result<()>;
    pub async fn allow_tool(&self, session_id: &str, tool_use_id: &str) -> Result<()>;
    pub async fn deny_tool(&self, session_id: &str, tool_use_id: &str, reason: impl Into<String>) -> Result<()>;
    pub async fn custom_tool_result(&self, session_id: &str, tool_use_id: &str, content: impl Into<String>) -> Result<()>;
    pub async fn define_outcome(&self, session_id: &str, criteria: impl Into<String>) -> Result<()>;

    // ─── Vault / Credentials (beta) ─────────────────────────────
    pub async fn create_vault(&self, params: CreateVaultParams) -> Result<Vault>;
    pub async fn list_vaults(&self) -> Result<ListResponse<Vault>>;
    pub async fn get_vault(&self, vault_id: &str) -> Result<Vault>;
    pub async fn archive_vault(&self, vault_id: &str) -> Result<()>;
    pub async fn delete_vault(&self, vault_id: &str) -> Result<()>;
    pub async fn create_credential(&self, vault_id: &str, params: CreateCredentialParams) -> Result<Credential>;
    pub async fn list_credentials(&self, vault_id: &str) -> Result<ListResponse<Credential>>;
    pub async fn update_credential(&self, vault_id: &str, cred_id: &str, params: UpdateCredentialParams) -> Result<Credential>;
    pub async fn validate_credential(&self, vault_id: &str, cred_id: &str) -> Result<CredentialValidation>;
    pub async fn delete_credential(&self, vault_id: &str, cred_id: &str) -> Result<()>;

    // ─── Memory (beta) ──────────────────────────────────────────
    pub async fn create_memory_store(&self, params: CreateMemoryStoreParams) -> Result<MemoryStore>;
    pub async fn list_memory_stores(&self) -> Result<ListResponse<MemoryStore>>;
    pub async fn get_memory_store(&self, store_id: &str) -> Result<MemoryStore>;
    pub async fn delete_memory_store(&self, store_id: &str) -> Result<()>;
    pub async fn create_memory(&self, store_id: &str, params: CreateMemoryParams) -> Result<Memory>;
    pub async fn list_memories(&self, store_id: &str) -> Result<ListResponse<Memory>>;
    pub async fn get_memory(&self, store_id: &str, memory_id: &str) -> Result<Memory>;
    pub async fn update_memory(&self, store_id: &str, memory_id: &str, params: UpdateMemoryParams) -> Result<Memory>;
    pub async fn delete_memory(&self, store_id: &str, memory_id: &str) -> Result<()>;
    pub async fn list_memory_versions(&self, store_id: &str) -> Result<ListResponse<MemoryVersion>>;
}
```

---

## Wire Types

The types are **shared** with `adk-managed` at the wire level. However, `adk-enterprise` defines its own copy (no dependency on `adk-managed`) because:
1. The client should never depend on the server crate
2. The client types include response-only fields (id, timestamps) that the runtime doesn't emit
3. Different `#[serde]` attributes may be needed (e.g., `#[serde(default)]` for optional response fields)

### Agent (Response Type)

```rust
/// A managed agent configuration as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,                       // "agt_..."
    pub name: String,
    pub model: ModelRef,
    pub system: Option<String>,
    pub description: Option<String>,
    pub tools: Vec<ToolConfig>,
    pub mcp_servers: Vec<McpServerConfig>,
    pub skills: Vec<SkillRef>,
    pub permission_policy: Option<PermissionPolicy>,
    pub metadata: Option<HashMap<String, String>>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}
```

### CreateAgentParams

```rust
/// Parameters for creating a new agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentParams {
    pub name: String,
    pub model: ModelRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<SkillRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_policy: Option<PermissionPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}
```

### Session

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,                       // "ses_..."
    pub agent_id: String,
    pub environment_id: Option<String>,
    pub status: SessionStatus,
    pub title: Option<String>,
    pub usage: Option<Usage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: Option<f64>,
}
```

### SessionEvent and UserEvent

Identical wire shapes to `adk-managed` (CANON §3.4). See the managed runtime spec for full definitions.

### Pagination

```rust
/// Cursor-paginated list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Parameters for list endpoints.
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
```

---

## SSE Streaming with Auto-Reconnect

```rust
/// A resilient event stream that auto-reconnects on disconnect.
///
/// Uses the `id:` field from SSE frames (which carries the `seq`) to
/// reconnect from the last received event via `Last-Event-ID` header.
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = Result<SessionEvent>> + Send>>,
}

impl EventStream {
    /// Get the underlying stream.
    pub fn into_stream(self) -> impl Stream<Item = Result<SessionEvent>> + Send;
}

// Implements Stream directly for ergonomic use:
impl Stream for EventStream {
    type Item = Result<SessionEvent>;
    fn poll_next(...) -> Poll<Option<Self::Item>>;
}
```

Auto-reconnect logic:
1. Track the last received `seq` from each event's `id:` SSE field
2. On disconnect or timeout, reconnect with `Last-Event-ID: {last_seq}` header
3. Server replays events with seq > last_seq
4. Transparent to the consumer — no gap in the stream

---

## Error Handling

```rust
/// Errors from the Enterprise API client.
#[derive(Debug, thiserror::Error)]
pub enum EnterpriseError {
    /// 400 — invalid request parameters.
    #[error("invalid request: {message}")]
    InvalidRequest { message: String, param: Option<String> },

    /// 401 — missing or invalid API key.
    #[error("authentication failed: {message}")]
    Authentication { message: String },

    /// 403 — insufficient permissions.
    #[error("permission denied: {message}")]
    Permission { message: String },

    /// 404 — resource not found.
    #[error("not found: {message}")]
    NotFound { message: String },

    /// 409 — conflict (e.g., invalid state transition).
    #[error("conflict: {message}")]
    Conflict { message: String },

    /// 422 — validation error.
    #[error("validation error: {message}")]
    Validation { message: String },

    /// 429 — rate limited.
    #[error("rate limited: retry after {retry_after:?}")]
    RateLimit { message: String, retry_after: Option<Duration> },

    /// 500 — internal server error.
    #[error("internal error: {message}")]
    Internal { message: String },

    /// 503 — service unavailable.
    #[error("service unavailable: {message}")]
    Unavailable { message: String, retry_after: Option<Duration> },

    /// Network/connection error.
    #[error("connection error: {0}")]
    Connection(#[from] reqwest::Error),

    /// SSE stream error.
    #[error("stream error: {message}")]
    Stream { message: String },

    /// SSE stream timeout (no data within configured duration).
    #[error("stream timeout after {timeout_secs}s")]
    StreamTimeout { timeout_secs: u64 },

    /// JSON serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

---

## Retry Policy

```rust
/// Automatic retry for transient errors (429, 500, 502, 503, 504).
///
/// Uses exponential backoff with jitter. Respects `Retry-After` header
/// on 429 responses. Non-retryable errors (400, 401, 403, 404) are
/// returned immediately.
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
}
```

---

## Request Headers

Every request includes:
```
Authorization: Bearer adk_live_...
ADK-Managed-Agent: 2026-06-01
Content-Type: application/json
```

Optional:
```
Idempotency-Key: <uuid>           (on create endpoints)
ADK-Beta: managed-agents-2026-06-01  (for beta features: vault, memory)
```

---

## Cargo.toml (Lightweight Dependencies)

```toml
[package]
name = "adk-enterprise"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "Native Rust SDK for ADK-Rust Enterprise Managed Agent Service"

[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { workspace = true }
serde_json = { workspace = true }
futures = { workspace = true }
tokio = { workspace = true, features = ["time", "sync"] }
thiserror = { workspace = true }
tracing = { workspace = true }
async-trait = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
bytes = "1"
async-stream = { workspace = true }
base64 = "0.22"

# NO dependency on adk-model, adk-runner, adk-agent, adk-session, etc.
# This is intentionally lightweight.
```

Note: **Zero dependency** on any `adk-*` crate. The types are self-contained. This means:
- Fast compile (no LLM provider code)
- Small binary (no agent runtime)
- Deployable anywhere (WASM-compatible with appropriate reqwest backend)

---

## Usage Examples

### Hello World

```rust
use adk_enterprise::{EnterpriseClient, UserEvent, SessionEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = EnterpriseClient::from_env()?;

    let agent = client.create_agent(CreateAgentParams {
        name: "Hello Agent".into(),
        model: "gemini-2.5-flash".into(),
        system: Some("You are brief and friendly.".into()),
        ..Default::default()
    }).await?;

    let session = client.create_session(&agent.id, None).await?;

    // Open stream BEFORE sending (required ordering)
    let mut stream = client.stream_events(&session.id).await?;
    client.send_message(&session.id, "What is 2+2?").await?;

    while let Some(event) = stream.next().await {
        match event? {
            SessionEvent::Message { content, .. } => {
                for block in content {
                    if let ContentBlock::Text { text } = block {
                        println!("{text}");
                    }
                }
            }
            SessionEvent::StatusIdle { .. } => break,
            _ => {}
        }
    }

    client.archive_session(&session.id).await?;
    Ok(())
}
```

### Custom Tool Flow

```rust
use adk_enterprise::*;
use futures::StreamExt;

let client = EnterpriseClient::from_env()?;

let agent = client.create_agent(CreateAgentParams {
    name: "Tool Agent".into(),
    model: "gpt-4.1".into(),
    tools: vec![
        ToolConfig::builtin("bash"),
        ToolConfig::custom("get_weather", "Get weather for a city", json!({
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        })),
    ],
    ..Default::default()
}).await?;

let session = client.create_session(&agent.id, None).await?;
let mut stream = client.stream_events(&session.id).await?;
client.send_message(&session.id, "What's the weather in Tokyo?").await?;

while let Some(event) = stream.next().await {
    match event? {
        SessionEvent::CustomToolUse { custom_tool_use_id, name, input, .. } => {
            // Execute the tool locally
            let result = match name.as_str() {
                "get_weather" => format!("22°C, sunny in {}", input["city"]),
                _ => "unknown tool".into(),
            };
            client.custom_tool_result(&session.id, &custom_tool_use_id, &result).await?;
        }
        SessionEvent::Message { content, .. } => { /* print response */ }
        SessionEvent::StatusIdle { .. } => break,
        _ => {}
    }
}
```

### Self-Hosted

```rust
// Point at your own deployment
let client = EnterpriseClient::self_hosted(
    "adk_live_your_key",
    "https://your-server.internal/managed/v1",
)?;

// Same API — everything works identically
let agent = client.create_agent(...).await?;
```

### Provider Switching

```rust
// Same agent definition, different model — same event stream shape
let params = CreateAgentParams {
    name: "My Agent".into(),
    system: Some("You are helpful.".into()),
    tools: vec![ToolConfig::builtin("web_search")],
    ..Default::default()
};

// Gemini
let gemini_agent = client.create_agent(params.clone().with_model("gemini-2.5-flash")).await?;

// OpenAI
let openai_agent = client.create_agent(params.clone().with_model(ModelRef::structured(
    Provider::Openai, "gpt-4.1"
))).await?;

// Anthropic
let claude_agent = client.create_agent(params.clone().with_model("claude-sonnet-4-6")).await?;

// DeepSeek (OpenAI-compatible)
let deepseek_agent = client.create_agent(params.with_model(ModelRef::compatible(
    "deepseek-chat",
    "https://api.deepseek.com",
))).await?;
```

---

## Side-by-Side: adk-enterprise vs adk-anthropic::managed_agents

| Operation | `adk-enterprise` | `adk-anthropic::managed_agents` |
|-----------|------------------|--------------------------------|
| **Create client** | `EnterpriseClient::new(key)` | `ManagedAgentsClient::new(key)` |
| **From env** | `EnterpriseClient::from_env()` (ADK_API_KEY) | `ManagedAgentsClient::from_env()` (ANTHROPIC_API_KEY) |
| **Create agent** | `client.create_agent(params)` | `client.create_agent(params)` |
| **Create env** | `client.create_environment(params)` | `client.create_environment(params)` |
| **Create session** | `client.create_session(agent_id, env_id)` | `client.create_session(params)` |
| **Stream events** | `client.stream_events(session_id)` | `client.stream_events(session_id)` |
| **Send message** | `client.send_message(session_id, text)` | `client.send_event(session_id, UserEvent::message(text))` |
| **Custom tool result** | `client.custom_tool_result(sid, tid, content)` | `client.custom_tool_result(sid, tid, content)` |
| **Allow tool** | `client.allow_tool(sid, tid)` | `client.allow_tool(sid, tid)` |
| **Deny tool** | `client.deny_tool(sid, tid, reason)` | `client.deny_tool(sid, tid, reason)` |
| **Interrupt** | `client.interrupt(session_id)` | `client.interrupt(session_id)` |
| **Archive session** | `client.archive_session(session_id)` | `client.archive_session(session_id)` |
| **Vault CRUD** | `client.create_vault(...)` | `client.create_vault(...)` |
| **Memory CRUD** | `client.create_memory_store(...)` | `client.create_memory_store(...)` |
| **Auto-reconnect** | ✅ Built-in (Last-Event-ID) | ❌ Manual |
| **Retry policy** | ✅ Configurable backoff | ❌ None |
| **Provider choice** | ✅ Any model via ModelRef | ❌ Claude only |
| **Self-hostable** | ✅ `self_hosted(key, url)` | ❌ Anthropic only |
| **Skills** | ✅ SkillRef in agent def | ❌ Not available |
| **Version header** | `ADK-Managed-Agent: 2026-06-01` | `anthropic-version: 2023-06-01` |

---

## What's Different (Our Advantages)

1. **Provider-neutral ModelRef** — `"gemini-2.5-flash"` or `{provider: "openai", model: "gpt-4.1"}` or `{provider: "openai_compatible", model: {model, base_url}}`
2. **Auto-reconnect SSE** — tracks `seq` via SSE `id:` field, reconnects transparently
3. **Configurable retry** — exponential backoff with jitter, respects `Retry-After`
4. **Self-hosted support** — same SDK works against your own deployment
5. **Skills** — attach Agent Skills packages to agents
6. **Pagination** — cursor-based with `has_more` (Anthropic returns flat arrays)
7. **Stop reason** — `status.idle` includes WHY the turn ended

---

## Timeline

| Phase | When | What |
|-------|------|------|
| **Phase 1** | After platform routes stabilize | Types + client struct + agent/session CRUD |
| **Phase 2** | After SSE endpoint is live | Stream processing + auto-reconnect |
| **Phase 3** | After beta features launch | Vault, memory, dreams |
| **Phase 4** | GA | Published to crates.io |

The SDK is generated/verified against the OpenAPI spec (`openapi/managed-agents.yaml`). When the OpenAPI changes, the SDK types are regenerated.
