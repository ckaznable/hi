## ADDED Requirements

### Requirement: Model pool with lazy initialization
The system SHALL provide a `ModelPool` that creates provider clients and agents on-demand (lazy), not at startup.

#### Scenario: First use of a model
- **WHEN** a model is requested for the first time via `ModelPool`
- **THEN** the system SHALL create the provider client and agent at that point

#### Scenario: Subsequent use of same model
- **WHEN** the same model is requested again and the previous agent is still alive
- **THEN** the system SHALL reuse the existing agent without creating a new one

### Requirement: Provider client sharing
The system SHALL share a single provider client instance across all agents using the same provider and API key.

#### Scenario: Two models same provider
- **WHEN** the default model and small_model both use `provider: "openai"`
- **THEN** the system SHALL create only one `openai::Client` and share it via `Arc`

### Requirement: Automatic resource reclamation
The system SHALL use `Weak` references for agents so they are automatically deallocated when no longer in use.

#### Scenario: Agent no longer referenced
- **WHEN** all strong references to an agent are dropped (e.g., heartbeat task finishes)
- **THEN** the agent SHALL be automatically deallocated

#### Scenario: Agent reclaimed and re-requested
- **WHEN** an agent was reclaimed and is requested again
- **THEN** the system SHALL create a new agent instance using the existing shared client

### Requirement: ModelRef resolution
The system SHALL resolve model references from config values:
- `"default"` or omitted → main model config
- `"small"` → `small_model` config
- inline object → custom model config

#### Scenario: Resolve "small" model ref
- **WHEN** a feature config has `model: "small"`
- **THEN** the system SHALL resolve it to the `small_model` configuration

#### Scenario: Resolve default model ref
- **WHEN** a feature config omits the `model` field
- **THEN** the system SHALL resolve it to the main model configuration

#### Scenario: No small_model configured but referenced
- **WHEN** a feature references `"small"` but `small_model` is not configured
- **THEN** the system SHALL fall back to the main model
