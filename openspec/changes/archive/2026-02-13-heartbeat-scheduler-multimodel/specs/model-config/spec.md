## MODIFIED Requirements

### Requirement: Config schema
The `ModelConfig` struct SHALL contain the following fields:
- `provider`: enum — one of `openai`, `anthropic`, `gemini`, `ollama`
- `model`: string — the model identifier (e.g. `"gpt-4o"`, `"claude-3-5-sonnet"`)
- `api_key`: optional string — required for all providers except `ollama`
- `api_base`: optional string — custom API endpoint override
- `preamble`: optional string — system prompt for the agent
- `context_window`: integer — maximum token count for the model's context window
- `small_model`: optional object — lightweight model config (same structure as top-level minus `small_model`, `heartbeat`, `schedules`)
- `heartbeat`: optional object — heartbeat configuration (`enabled`, `interval_secs`, `model`, `prompt`)
- `schedules`: optional array — scheduled task definitions (`name`, `cron`, `model`, `prompt`)

#### Scenario: Config with small_model
- **WHEN** config JSON includes `"small_model": { "provider": "ollama", "model": "qwen2.5:3b", "context_window": 4096 }`
- **THEN** the system SHALL deserialize it as a secondary model configuration

#### Scenario: Config without small_model
- **WHEN** config JSON omits the `small_model` field
- **THEN** the system SHALL use the main model as fallback for all `"small"` references

#### Scenario: Config with heartbeat
- **WHEN** config JSON includes `"heartbeat": { "enabled": true, "interval_secs": 300, "model": "small", "prompt": "Status check" }`
- **THEN** the system SHALL deserialize the heartbeat configuration

#### Scenario: Config with schedules
- **WHEN** config JSON includes a `"schedules"` array with task entries
- **THEN** the system SHALL deserialize each scheduled task configuration
