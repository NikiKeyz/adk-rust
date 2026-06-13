# Realtime Voice — Mindfulness with Mia

A full **web UI** demonstrating ADK-Rust real-time voice — a mindfulness coaching
app where the **Rust server owns the realtime session** through
[`IntegratedRealtimeRunner`], so transcripts, memory, and tool execution all
happen server-side. The browser is a thin audio device.

## What This Shows

| Capability | Description |
|-----------|-------------|
| **Full Web UI** | Browser-based voice coaching interface served by Axum |
| **Server-side bridge** | The Rust server owns the realtime session via `IntegratedRealtimeRunner`; the browser only captures/plays audio |
| **Audio capture** | Web Audio API microphone capture at 24 kHz PCM16, streamed to the server over WebSocket |
| **Audio playback** | Gapless Web Audio playback of the assistant's PCM stream, with barge-in |
| **Session persistence** | Completed turns persisted to a `SessionService` |
| **Knowledge-graph memory** | A file-backed `GraphMemoryService` (bi-temporal KG) is Mia's long-term memory: her profile card is injected into the system prompt at session start, and every turn is logged to the graph's episodic store |
| **Live memory panel** | The "User Memory Insights" panel reads and writes the **same** graph over `/api/memory` — add a fact, reset to baseline, all server-side |
| **Tool calling** | `get_weather` executed **server-side**; the result is fed back to the model |
| **VAD** | Server-side voice activity detection for natural turn-taking |
| **Coaching persona** | "Mia" mindfulness coach with guidelines and preferences |

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                      Browser (Web UI)                         │
│   mic ──PCM16 (base64 over WS)──▶        ◀── assistant PCM16  │
│   Web Audio capture @ 24 kHz             Web Audio playback   │
└───────────┬───────────────────────────────────────▲──────────┘
            │ WebSocket /ws                          │
┌───────────▼───────────────────────────────────────┴──────────┐
│     Axum server (localhost:3033)                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  IntegratedRealtimeRunner                            │    │
│  │  ├─ OpenAIRealtimeModel  (gpt-realtime, voice "marin") │    │
│  │  ├─ SessionService     → transcript persistence       │    │
│  │  ├─ GraphMemoryService → bi-temporal KG (shared)       │    │
│  │  └─ get_weather tool   → auto-executed, result returned│    │
│  └──────────────────────────────────────────────────────┘    │
│         ▲ profile card injected into the prompt;  ▲           │
│         └ turns logged to episodic store          └ /api/memory│
└──────────────────────────────────────────────────────────────┘
```

The server never exposes your API key to the browser — the realtime connection
to OpenAI lives entirely on the server side.

## Memory — a real knowledge graph

Mia's memory is a single, process-wide
[`GraphMemoryService`](../../adk-memory/src/graph.rs): a SQLite-backed,
**bi-temporal knowledge graph** (entities, typed relations, time-stamped
observations). It is shared between the realtime bridge and the web UI, so the
agent and the "User Memory Insights" panel are looking at the *same* memory:

- **On connect**, the server reads the graph's compact *profile card* and bakes
  it into Mia's system instruction — that's how she greets Shai already knowing
  he relocated to the Bay Area and prefers to be addressed by name. (Nothing on
  the Insights panel is mocked; it's rendered from the graph.)
- **During the session**, every completed turn is appended to the graph's
  episodic log via the integration layer's `store_to_memory`.
- **The panel** reads `GET /api/memory` and writes `POST /api/memory`
  (add an observation under a category/entity) and `POST /api/memory/reset`
  (wipe and re-seed the baseline profile).

The graph is file-backed (`mia_memory.db` by default, override with
`MIA_MEMORY_DB`), so Mia remembers Shai across restarts. On first run an empty
graph is seeded with Shai's baseline profile.

> Heavy fact-extraction (distilling raw turns into new entities/relations) is
> intentionally left to an out-of-band consolidation pass rather than the hot
> path — see the `GraphMemoryService` docs. This example seeds the profile and
> lets you curate it from the panel; it does not yet auto-extract facts from the
> conversation.

## Providers

Pick **OpenAI** or **Gemini** from the dropdown before starting a session — the
browser passes the choice to the server (`/ws?provider=…`), which builds the
matching realtime model. Because their audio rates differ (OpenAI 24 kHz in/out;
Gemini Live 16 kHz in / 24 kHz out), the server negotiates the sample rates to
the browser in a `ready` message before any audio flows, and the browser
configures its capture/playback contexts accordingly.

## Prerequisites

- Rust 1.94.0+
- `OPENAI_API_KEY` (for OpenAI) and/or `GEMINI_API_KEY` / `GOOGLE_API_KEY` (for Gemini)
- A modern browser with WebSocket + Web Audio API support and microphone access

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `OPENAI_API_KEY` | For OpenAI | OpenAI API key |
| `GEMINI_API_KEY` / `GOOGLE_API_KEY` | For Gemini | Google AI Studio key |
| `OPENAI_REALTIME_MODEL` | No | OpenAI model ID (default: `gpt-realtime`; `gpt-realtime-2` for the reasoning model) |
| `GEMINI_REALTIME_MODEL` | No | Gemini model ID (default: `models/gemini-3.1-flash-live-preview`, which calls tools reliably; `models/gemini-2.5-flash-native-audio-preview-12-2025` for the most natural voice) |
| `MIA_MEMORY_DB` | No | Knowledge-graph SQLite path (default: `mia_memory.db`) |
| `PORT` | No | Server port (default: `3033`) |
| `RUST_LOG` | No | Log level (default: `info`) |

> **Gemini model note:** AI Studio (API-key) uses different model names than
> Vertex/Agent Platform. The half-cascade `gemini-3.1-flash-live-preview` is the
> default here because the native-audio model, while more natural-sounding, calls
> tools far less reliably.

## Run

```bash
# Web UI
cargo run --manifest-path examples/realtime_voice/Cargo.toml
# → open http://localhost:3033

# Headless smoke test (no browser/mic) — connects, asks a weather question by
# text, verifies the tool runs and audio comes back. Pick a provider:
cargo run --manifest-path examples/realtime_voice/Cargo.toml -- probe openai
cargo run --manifest-path examples/realtime_voice/Cargo.toml -- probe gemini
```

## How It Works

1. Click **START VOICE SESSION** — the browser opens a WebSocket to the server.
2. The server builds an `IntegratedRealtimeRunner` (the chosen model + an
   in-memory `SessionService` + the shared `GraphMemoryService` + the
   `get_weather` tool), injects the graph's profile card into Mia's
   instruction, and connects.
3. The browser captures microphone audio as 24 kHz PCM16 and streams base64
   frames up the WebSocket.
4. Server VAD detects turn boundaries; the model responds automatically.
5. The runner streams the assistant's PCM audio + transcript back to the browser,
   which plays it gaplessly via Web Audio. Barge-in flushes playback.
6. When the model calls `get_weather`, the runner executes it **server-side** and
   returns the result so Mia can speak it.
7. Each completed turn is persisted to the session and appended to the
   knowledge graph's episodic log.

## UI

- **Left panel** — avatar, voice controls (start / mute / pause / hang up), the
  active memory cache (top facts from the graph), coaching guidelines, MIA/USER
  status.
- **Right panel** — **User Memory Insights** (rendered from the live knowledge
  graph; add a fact or reset to baseline), Coaching Strategy, and a live
  Pipeline Decisions log (tool calls and session events).

## Feature Flags

```toml
adk-realtime = { version = "1.1.0", features = ["openai", "gemini", "integration"] }
adk-memory   = { version = "1.1.0", features = ["graph-memory"] }
```
