## ADDED Requirements

### Requirement: OpenAI-compatible provider mode
The system SHALL support an `openai-compatible` provider mode that targets OpenAI chat-completions compatible endpoints using configurable `model` and `api_base` values.

#### Scenario: Deserialize openai-compatible provider
- **WHEN** config JSON contains `"provider": "openai-compatible"`
- **THEN** the system SHALL deserialize it into the provider enum variant for OpenAI-compatible mode

#### Scenario: Build agent for openai-compatible endpoint
- **WHEN** provider is `openai-compatible` and config includes `api_base`
- **THEN** the system SHALL construct the chat agent using the OpenAI client path and send requests to the configured base URL

### Requirement: OpenAI-compatible authentication behavior
The system SHALL allow `api_key` to be omitted in `openai-compatible` mode, while still accepting `api_key` when provided.

#### Scenario: Openai-compatible config without api_key
- **WHEN** config JSON contains `{"provider": "openai-compatible", "model": "gpt-4o-mini", "api_base": "http://localhost:11434/v1", "context_window": 32000}` and omits `api_key`
- **THEN** the system SHALL pass config validation

#### Scenario: Openai-compatible config with api_key
- **WHEN** config JSON contains `{"provider": "openai-compatible", "model": "gpt-4o-mini", "api_key": "test-key", "api_base": "https://gateway.example.com/v1", "context_window": 32000}`
- **THEN** the system SHALL pass config validation and use the provided key for client construction
