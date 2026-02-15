## ADDED Requirements

### Requirement: Init command generates config template
The CLI SHALL provide an `init` subcommand that creates a starter `config.json` template at the default config path `ProjectDirs::config_dir("hi")/config.json`.

#### Scenario: Config template created at default path
- **WHEN** a user runs `hi init` and `config.json` does not exist in the default config directory
- **THEN** the system SHALL create the config directory if needed and write `config.json` at the default path

### Requirement: Generated template is valid minimal model config
The generated `config.json` SHALL be valid JSON and SHALL include at least the fields required for minimal startup configuration: `provider`, `model`, `api_key`, and `context_window`.

#### Scenario: Template contains required fields
- **WHEN** `hi init` successfully creates `config.json`
- **THEN** the file content SHALL parse as JSON and include `provider`, `model`, `api_key`, and `context_window`

#### Scenario: Template is accepted by config loader
- **WHEN** `hi init` has created `config.json` and the application later loads configuration from the default path
- **THEN** the system SHALL parse the generated template as a valid `ModelConfig`

### Requirement: Existing config is not overwritten
The `init` command SHALL NOT overwrite an existing `config.json` at the target path.

#### Scenario: Existing config file
- **WHEN** a user runs `hi init` and `config.json` already exists at the default path
- **THEN** the system SHALL return an error indicating the file already exists and SHALL keep the existing file unchanged

### Requirement: Init command reports actionable outcomes
The `init` command SHALL provide user-facing output for both success and failure outcomes, including the resolved config path when applicable.

#### Scenario: Successful initialization output
- **WHEN** `hi init` creates `config.json`
- **THEN** the system SHALL print a success message that includes the resolved config file path

#### Scenario: Filesystem failure output
- **WHEN** `hi init` cannot create the config directory or write `config.json` due to filesystem errors (for example permission denied)
- **THEN** the system SHALL return an error message that includes the target path and the underlying failure reason
