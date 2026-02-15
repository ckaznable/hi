## ADDED Requirements

### Requirement: Config file location
The system SHALL read the model configuration from a JSON file located at `ProjectDirs::config_dir("hi")/config.json`.

#### Scenario: Config file exists
- **WHEN** the application starts and `config.json` exists in the config directory
- **THEN** the system SHALL parse the file and return a valid `ModelConfig`

#### Scenario: Config file does not exist
- **WHEN** the application starts and `config.json` does not exist
- **THEN** the system SHALL return an error with a message indicating the expected config file path

### Requirement: Config schema
The `ModelConfig` struct SHALL contain the following fields:
- `provider`: enum — one of `openai`, `anthropic`, `gemini`, `ollama`
- `model`: string — the model identifier (e.g. `"gpt-4o"`, `"claude-3-5-sonnet"`)
- `api_key`: optional string — required for all providers except `ollama`
- `api_base`: optional string — custom API endpoint override
- `preamble`: optional string — system prompt for the agent
- `context_window`: integer — maximum token count for the model's context window

#### Scenario: Valid OpenAI config
- **WHEN** config JSON contains `{"provider": "openai", "model": "gpt-4o", "api_key": "sk-...", "context_window": 128000}`
- **THEN** the system SHALL deserialize it into a `ModelConfig` with `Provider::OpenAI`

#### Scenario: Ollama config without api_key
- **WHEN** config JSON contains `{"provider": "ollama", "model": "qwen2.5:14b", "context_window": 32000}` without `api_key`
- **THEN** the system SHALL deserialize it successfully

#### Scenario: Non-ollama config without api_key
- **WHEN** config JSON contains `{"provider": "openai", "model": "gpt-4o", "context_window": 128000}` without `api_key`
- **THEN** the system SHALL return a validation error indicating `api_key` is required

### Requirement: Provider enum
The `Provider` enum SHALL support the following variants: `OpenAI`, `Anthropic`, `Gemini`, `Ollama`. The enum SHALL deserialize from lowercase JSON strings.

#### Scenario: Deserialize provider
- **WHEN** the JSON `provider` field is `"anthropic"`
- **THEN** the system SHALL deserialize it as `Provider::Anthropic`

#### Scenario: Unknown provider
- **WHEN** the JSON `provider` field is `"unknown_provider"`
- **THEN** the system SHALL return a deserialization error
