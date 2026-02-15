## ADDED Requirements

### Requirement: Config validation command probes configured model
The CLI SHALL provide a config validation command that sends a minimal `hi` prompt to the primary model configured in `config.json` using the same provider client path as normal runtime requests.

#### Scenario: Probe succeeds
- **WHEN** a user runs the config validation command and the configured provider endpoint, credentials, and model are valid
- **THEN** the system SHALL send one probe request with user content `hi` and SHALL report validation success

#### Scenario: Probe request shape matches runtime path
- **WHEN** the config validation command executes
- **THEN** the system SHALL construct the model request through the existing runtime provider selection and client initialization flow instead of a separate probe-only transport path

### Requirement: Validation command produces actionable outcome messages
The config validation command SHALL return user-facing output that distinguishes success from common failure classes.

#### Scenario: Authentication failure
- **WHEN** the provider rejects the probe due to invalid or missing credentials
- **THEN** the system SHALL report an authentication-related validation failure with guidance to check configured credentials

#### Scenario: Endpoint or network failure
- **WHEN** the probe cannot reach the configured endpoint because of connection, DNS, TLS, or timeout errors
- **THEN** the system SHALL report an endpoint/network validation failure with guidance to check `api_base` and network reachability

#### Scenario: Model not available
- **WHEN** the provider returns an error indicating the configured model does not exist or is unavailable
- **THEN** the system SHALL report a model-related validation failure with guidance to check the configured model identifier
