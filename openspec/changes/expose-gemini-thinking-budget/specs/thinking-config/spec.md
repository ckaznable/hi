## ADDED Requirements

### Requirement: Thinking configuration
The system SHALL support configuring extended thinking options via the `thinking` field in model config.

#### Scenario: Thinking enabled with budget
- **WHEN** config has `thinking: { type: "enabled", budget_tokens: 1024 }`
- **THEN** the system SHALL pass thinking configuration to the LLM provider

#### Scenario: Thinking disabled
- **WHEN** config has no `thinking` field or `thinking: null`
- **THEN** the system SHALL not pass thinking configuration (provider default behavior)

#### Scenario: Thinking with auto type
- **WHEN** config has `thinking: { type: "auto" }`
- **THEN** the system SHALL let the provider decide whether to use thinking
