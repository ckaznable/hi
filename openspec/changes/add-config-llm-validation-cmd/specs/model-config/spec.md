## ADDED Requirements

### Requirement: Configured model can be explicitly validated from CLI
The system SHALL allow users to validate the current primary `ModelConfig` by executing a dedicated CLI command that performs a live probe against the configured provider and model.

#### Scenario: Validate current default config file
- **WHEN** a user runs the config validation command without overriding config path and the default config file is loadable
- **THEN** the system SHALL validate the currently loaded primary model configuration by issuing one live probe request

#### Scenario: Validation does not mutate configuration
- **WHEN** the config validation command runs against a valid or invalid model configuration
- **THEN** the system SHALL NOT rewrite or mutate config file contents as part of validation

### Requirement: Validation failure categorization for model configuration errors
When model configuration is present but cannot be used for successful probe execution, the system SHALL surface categorized validation failures to support operator troubleshooting.

#### Scenario: Invalid configuration for selected provider
- **WHEN** config values are incompatible with the selected provider requirements (for example missing required credentials)
- **THEN** the system SHALL return a validation failure that identifies configuration incompatibility for the selected provider

#### Scenario: Configuration points to unreachable service
- **WHEN** the config contains an endpoint that cannot be reached at validation time
- **THEN** the system SHALL return a validation failure that identifies reachability problems for the configured endpoint
