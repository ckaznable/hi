## MODIFIED Requirements

### Requirement: Config schema
The `ModelConfig` struct SHALL contain the following fields:
- `provider`: enum — one of `openai`, `openai-compatible`, `anthropic`, `gemini`, `ollama`
- `model`: string — the model identifier (e.g. `"gpt-4o"`, `"claude-3-5-sonnet"`)
- `api_key`: optional string — required for `openai`, `anthropic`, and `gemini`; optional for `openai-compatible` and `ollama`
- `api_base`: optional string — custom API endpoint override
- `preamble`: optional string — system prompt for the agent
- `context_window`: integer — maximum token count for the model's context window
- `small_model`: optional object — lightweight model config (same structure as top-level minus `small_model`, `heartbeat`, `schedules`)
- `heartbeat`: optional object — heartbeat configuration (`enabled`, `interval_secs`, `model`, `prompt`)
- `schedules`: optional array — scheduled task definitions (`name`, `cron`, `model`, `prompt`)

#### Scenario: Valid OpenAI config
- **WHEN** config JSON contains `{"provider": "openai", "model": "gpt-4o", "api_key": "sk-...", "context_window": 128000}`
- **THEN** the system SHALL deserialize it into a `ModelConfig` with `Provider::OpenAI`

#### Scenario: Valid openai-compatible config
- **WHEN** config JSON contains `{"provider": "openai-compatible", "model": "gpt-4o-mini", "api_base": "https://gateway.example.com/v1", "context_window": 64000}`
- **THEN** the system SHALL deserialize it into a `ModelConfig` with OpenAI-compatible provider semantics

#### Scenario: Ollama config without api_key
- **WHEN** config JSON contains `{"provider": "ollama", "model": "qwen2.5:14b", "context_window": 32000}` without `api_key`
- **THEN** the system SHALL deserialize it successfully

#### Scenario: Non-exempt provider without api_key
- **WHEN** config JSON contains `{"provider": "openai", "model": "gpt-4o", "context_window": 128000}` without `api_key`
- **THEN** the system SHALL return a validation error indicating `api_key` is required

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

### Requirement: Provider enum
The `Provider` enum SHALL support the following variants: `OpenAI`, `OpenAICompatible`, `Anthropic`, `Gemini`, `Ollama`. The enum SHALL deserialize from lowercase JSON strings.

#### Scenario: Deserialize provider
- **WHEN** the JSON `provider` field is `"anthropic"`
- **THEN** the system SHALL deserialize it as `Provider::Anthropic`

#### Scenario: Deserialize openai-compatible provider
- **WHEN** the JSON `provider` field is `"openai-compatible"`
- **THEN** the system SHALL deserialize it as `Provider::OpenAICompatible`

#### Scenario: Unknown provider
- **WHEN** the JSON `provider` field is `"unknown_provider"`
- **THEN** the system SHALL return a deserialization error
