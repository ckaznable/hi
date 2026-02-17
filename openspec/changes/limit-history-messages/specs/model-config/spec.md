## MODIFIED Requirements

### Requirement: Config schema (UPDATED)
The `ModelConfig` struct SHALL contain the following fields:
- `provider`: enum — one of `openai`, `anthropic`, `gemini`, `ollama`
- `model`: string — the model identifier
- `api_key`: optional string — required for all providers except `ollama`
- `api_base`: optional string — custom API endpoint override
- `preamble`: optional string — system prompt for the agent
- `context_window`: integer — maximum token count for the model's context window
- `small_model`: optional object — lightweight model config
- `heartbeat`: optional object — heartbeat configuration
- `schedules`: optional array — scheduled task definitions
- **`history_limit`**: optional integer — maximum number of history messages to send to LLM (default: unlimited, use 0 for no history)

#### Scenario: Config with history_limit
- **WHEN** config JSON includes `"history_limit": 10`
- **THEN** the system SHALL limit history to 10 most recent messages

#### Scenario: Config without history_limit
- **WHEN** config JSON omits the `history_limit` field
- **THEN** the system SHALL send all available history (default behavior)
