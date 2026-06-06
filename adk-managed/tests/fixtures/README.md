# Golden Fixture Conformance Test Corpus

This directory contains the golden fixture JSON files used to verify the managed agent runtime produces correct, deterministic event sequences.

## Schema

Each fixture file follows this unified schema:

```json
{
  "name": "F-N: Short name",
  "description": "What this fixture tests",
  "agent_def": {
    "name": "agent-name",
    "model": "gemini-2.5-flash",
    "system": "System prompt",
    "tools": [],
    "mcp_servers": [],
    "skills": []
  },
  "scripted_model": {
    "turns": [
      {
        "text": "Response text (optional)",
        "tool_calls": [
          {
            "name": "tool_name",
            "input": {},
            "id": "tc_001"
          }
        ]
      }
    ]
  },
  "scenario": [
    {
      "type": "user.message",
      "content": [{"type": "text", "text": "Hello"}]
    }
  ],
  "assertions": {
    "exact_sequence": ["status.running", "agent.message", "status.idle"],
    "must_contain": ["agent.message"],
    "must_end_with": ["status.idle"]
  }
}
```

## Fields

### `agent_def`

The declarative agent definition matching `ManagedAgentDef`. Used in both test modes.

### `scripted_model`

Contains `turns` — an array of `ScriptedTurn` objects that the `ScriptedLlm` double will return in FIFO order. Used only in `scripted` mode.

### `scenario`

An ordered list of `UserEvent` objects to send to the runtime. These drive the test execution.

### `assertions`

- **`exact_sequence`** (scripted mode): The complete, ordered list of `SessionEvent` type strings that MUST be produced. Byte-identical matching — no gaps, no extras.
- **`must_contain`** (real mode): Event types that MUST appear somewhere in the output.
- **`must_end_with`** (real mode): The final event(s) in the stream MUST match these types.

## Test Modes

Controlled by the `ADK_TEST_MODE` environment variable:

### `scripted` (default)

- Uses `ScriptedLlm` with `scripted_model.turns`
- Asserts `exact_sequence` — byte-identical type sequence
- Runs on every commit, blocks merge
- Cost: $0

### `real`

- Uses the model from `agent_def.model` with real API credentials
- Asserts `must_contain` + `must_end_with` (subsequence matching)
- Runs nightly or on-demand
- Cost: ~$0.05/run

## Fixtures

| File | Name | Tests |
|------|------|-------|
| `f1_hello.json` | Hello | Basic message → response → idle |
| `f2_mcp_tool.json` | MCP Tool | MCP tool call flow |
| `f3_custom_tool.json` | Custom Tool | Park → deliver → resume |
| `f4_confirmation.json` | Tool Confirmation | Confirmation request → approve → execute |
| `f5_resume.json` | Resume After Kill | Crash → resume from checkpoint |
| `f6_replay.json` | Replay from_seq | Historical event replay |
| `f7_interrupt.json` | Interrupt Mid-Turn | Interrupt signal stops at boundary |
| `f8_provider_parity.json` | Provider Parity | Same sequence across all providers |

## Running

```bash
# Scripted mode (default, per-commit gate)
cargo test -p adk-managed --test fixture_conformance_tests

# Real mode (nightly, requires API keys)
ADK_TEST_MODE=real cargo test -p adk-managed --test fixture_conformance_tests -- --ignored
```
