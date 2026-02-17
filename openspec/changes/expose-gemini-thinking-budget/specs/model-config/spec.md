## MODIFIED Requirements

### Requirement: Config schema (UPDATED)
The `ModelConfig` struct SHALL contain the following fields:
- `provider`: enum — one of `openai`, `anthropic`, `gemini`, `ollama`
- `model`: string — the model identifier (e.g. `"gpt-4o"`, `"claude-3-5-sonnet"`)
- `api_key`: optional string — required for all providers except `ollama`
- `api_base`: optional string — custom API endpoint override
- `preamble`: optional string — system prompt for the agent
- `context_window`: integer — maximum token count for the model's context window
- `small_model`: optional object — lightweight model config (same structure as top-level minus `small_model`, `heartbeat`, `schedules`, `thinking`)
- `heartbeat`: optional object — heartbeat configuration (`enabled`, `interval_secs`, `model`, `prompt`)
- `schedules`: optional array — scheduled task definitions (`name`, `cron`, `model`, `prompt`)
- **`thinking`**: optional object — thinking/extended thinking configuration (`type`, `budget_tokens`)

#### Scenario: Config with thinking
- **WHEN** config JSON includes `"thinking": { "type": "enabled", "budget_tokens": 1024 }`
- **THEN** the system SHALL deserialize it as a thinking configuration

#### Scenario: Config without thinking
- **WHEN** config JSON omits the `thinking` field
- **THEN** the system SHALL use default thinking behavior (provider-specific)
