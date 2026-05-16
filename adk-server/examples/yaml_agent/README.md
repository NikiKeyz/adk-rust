# YAML Agent Configuration Examples

This directory contains example YAML agent definition files demonstrating
the features of the `yaml-agent` module in `adk-server`.

## Features Demonstrated

### Environment Variable Interpolation

All string fields support `${VAR}` and `${VAR:-default}` placeholders:

```yaml
model:
  provider: "${LLM_PROVIDER:-gemini}"
  model_id: "${LLM_MODEL:-gemini-2.5-flash}"

session:
  backend: postgres
  connection_string: "${DATABASE_URL}"
```

- `${VAR}` — replaced with the environment variable value; error if unset
- `${VAR:-default}` — uses the default value if the variable is unset
- Interpolation is applied to all string fields recursively before validation
- All unresolved variables are reported at once (multi-error)

### Plugin References

Attach lifecycle plugins to agents by name:

```yaml
plugins:
  - name: telemetry
    config:
      endpoint: "${OTEL_ENDPOINT:-http://localhost:4317}"
  - name: rate_limit
    config:
      max_requests_per_minute: 60
```

### Session Configuration

Configure session persistence backends:

```yaml
session:
  backend: postgres          # inmemory, sqlite, postgres, redis
  connection_string: "${DATABASE_URL}"
  pool_size: 5
```

### Memory Configuration

Configure semantic memory backends:

```yaml
memory:
  backend: inmemory          # inmemory, postgres
```

## Files

- `full_example.yaml` — Complete agent definition using all features
- `minimal.yaml` — Minimal valid agent definition (name + model only)

## Running

```bash
# Set required environment variables
export LLM_PROVIDER=gemini
export LLM_MODEL=gemini-2.5-flash
export DATABASE_URL=postgres://localhost/mydb

# Load via the AgentConfigLoader API
cargo run -p adk-server --features yaml-agent
```

## Serialization Round-Trip

Agent definitions can be serialized back to YAML:

```rust
use adk_server::yaml_agent::{serialize_definition, YamlAgentDefinition};

let def: YamlAgentDefinition = /* loaded from file */;
let yaml_output = serialize_definition(&def)?;
// yaml_output can be written back to a file
```
