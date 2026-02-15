## MODIFIED Requirements

### Requirement: ModelRef resolution
The system SHALL resolve model references from config values:
- `"default"` or omitted -> main model config
- `"small"` -> `small_model` config
- inline object -> custom model config

Resolved model configurations SHALL preserve provider-specific fields (`provider`, `model`, `api_key`, `api_base`, `context_window`) so secondary features (heartbeat and schedules) can target any supported provider, including `openai-compatible`.

#### Scenario: Resolve "small" model ref
- **WHEN** a feature config has `model: "small"`
- **THEN** the system SHALL resolve it to the `small_model` configuration

#### Scenario: Resolve default model ref
- **WHEN** a feature config omits the `model` field
- **THEN** the system SHALL resolve it to the main model configuration

#### Scenario: No small_model configured but referenced
- **WHEN** a feature references `"small"` but `small_model` is not configured
- **THEN** the system SHALL fall back to the main model

#### Scenario: Resolve openai-compatible small model
- **WHEN** `small_model` is configured with `provider: "openai-compatible"` and a heartbeat or schedule references `model: "small"`
- **THEN** the system SHALL resolve to that OpenAI-compatible `small_model` including its `api_base` and auth fields

#### Scenario: Resolve inline openai-compatible model
- **WHEN** a feature config provides an inline `model` object with `provider: "openai-compatible"`
- **THEN** the system SHALL use the inline configuration directly for agent creation
