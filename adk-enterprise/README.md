# adk-enterprise

Native Rust SDK for the ADK-Rust Enterprise Managed Agent Service.

A lightweight, typed client for creating agents, managing sessions, sending messages, and streaming
responses over HTTP/SSE. Zero dependency on `adk-model`, `adk-runner`, `adk-agent`, or any heavy
runtime crate.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
adk-enterprise = { git = "https://github.com/zavora-ai/adk-rust" }
futures = "0.3"
tokio = { version = "1", features = ["full"] }
```

Or via the umbrella crate with the `enterprise` feature:

```toml
[dependencies]
adk-rust = { git = "https://github.com/zavora-ai/adk-rust", features = ["enterprise"] }
```

## Quick Start

```rust
use adk_enterprise::{EnterpriseClient, CreateAgentParams, SessionEvent, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create client from ADK_API_KEY env var
    let client = EnterpriseClient::from_env()?;

    // Create an agent with any model
    let agent = client.create_agent(CreateAgentParams {
        name: "Hello Agent".into(),
        model: "gemini-2.5-flash".into(),
        system: Some("You are brief and friendly.".into()),
        ..Default::default()
    }).await?;

    // Start a session
    let session = client.create_session(&agent.id, None).await?;

    // Open stream BEFORE sending (required ordering)
    let mut stream = client.stream_events(&session.id).await?;

    // Send a message
    client.send_message(&session.id, "What is 2+2?").await?;

    // Process events
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

    // Clean up
    client.archive_session(&session.id).await?;
    Ok(())
}
```

## Custom Tool Example

Handle custom tools that the agent invokes but you execute locally:

```rust
use adk_enterprise::*;
use futures::StreamExt;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = EnterpriseClient::from_env()?;

    let agent = client.create_agent(CreateAgentParams {
        name: "Tool Agent".into(),
        model: "gpt-4.1".into(),
        tools: vec![
            ToolConfig::builtin("bash"),
            ToolConfig::custom("get_weather", Some("Get weather for a city"), json!({
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
                let result = match name.as_str() {
                    "get_weather" => format!("22°C, sunny in {}", input["city"]),
                    _ => "unknown tool".into(),
                };
                client.custom_tool_result(&session.id, &custom_tool_use_id, &result).await?;
            }
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

## Self-Hosted Example

Point the SDK at your own deployment — same API, different URL:

```rust
use adk_enterprise::EnterpriseClient;

let client = EnterpriseClient::self_hosted(
    "adk_live_your_key",
    "https://your-server.internal/managed/v1",
)?;

// Everything works identically
let agents = client.list_agents(None).await?;
```

## Features

- **Zero adk-\* dependencies** — only HTTP + SSE. Fast compile, small binary.
- **Any model** — use Gemini, OpenAI, Anthropic, DeepSeek, or any OpenAI-compatible provider via `ModelRef`.
- **Auto-reconnect SSE** — tracks `seq` via SSE `id:` field, reconnects transparently with `Last-Event-ID`.
- **Automatic retry** — exponential backoff with jitter on 429/5xx. Respects `Retry-After` headers.
- **Idempotency** — UUID v4 keys on all create operations, reused across retries.
- **Self-hosted support** — same SDK works against any deployment URL.
- **Forward-compatible** — unknown event types deserialize to `SessionEvent::Unknown`.
- **Clone + Send + Sync** — share the client across tasks without `Arc`.
- **Typed errors** — `EnterpriseError` variants for every failure mode with `is_retryable()`.
- **Beta features** — Vault/Credential management and Memory stores (with beta header).

## API Overview

| Category | Methods |
|----------|---------|
| **Construction** | `new(key)`, `from_env()`, `self_hosted(key, url)`, `with_config(config)` |
| **Agents** | `create_agent`, `get_agent`, `list_agents`, `update_agent`, `archive_agent`, `delete_agent` |
| **Environments** | `create_environment`, `get_environment`, `archive_environment`, `delete_environment`, `download_environment` |
| **Sessions** | `create_session`, `create_session_full`, `get_session`, `list_sessions`, `pause_session`, `resume_session`, `archive_session`, `delete_session` |
| **Events** | `send_event`, `stream_events`, `list_events` |
| **Convenience** | `send_message`, `interrupt`, `allow_tool`, `deny_tool`, `custom_tool_result`, `define_outcome` |
| **Vault (beta)** | `create_vault`, `list_vaults`, `get_vault`, `archive_vault`, `delete_vault`, `create_credential`, `list_credentials`, `update_credential`, `validate_credential`, `delete_credential` |
| **Memory (beta)** | `create_memory_store`, `list_memory_stores`, `get_memory_store`, `delete_memory_store`, `create_memory`, `list_memories`, `get_memory`, `update_memory`, `delete_memory`, `list_memory_versions` |

## Error Handling

All methods return `Result<T, EnterpriseError>`. The error type maps directly to API responses:

```rust
use adk_enterprise::{EnterpriseClient, EnterpriseError};

match client.get_agent("agt_nonexistent").await {
    Ok(agent) => println!("Found: {}", agent.name),
    Err(EnterpriseError::NotFound { message }) => println!("Agent not found: {message}"),
    Err(EnterpriseError::Authentication { message }) => println!("Bad API key: {message}"),
    Err(EnterpriseError::RateLimit { retry_after, .. }) => {
        println!("Rate limited, retry after {retry_after:?}");
    }
    Err(e) if e.is_retryable() => println!("Transient error (already retried): {e}"),
    Err(e) => println!("Permanent error: {e}"),
}
```

## Configuration

```rust
use adk_enterprise::ClientConfig;
use std::time::Duration;

let config = ClientConfig::new("adk_live_...")
    .with_base_url("https://staging.enterprise.adk-rust.com/managed/v1")
    .with_sse_timeout(Duration::from_secs(600))
    .with_max_retries(5);

let client = EnterpriseClient::with_config(config)?;
```

## Requirements

- **Rust 1.85+** (edition 2024)
- **tokio** async runtime
- Network access to the Enterprise platform (or self-hosted deployment)
- API key (`adk_live_...` or `adk_test_...`)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.
