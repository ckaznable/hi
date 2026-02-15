## MODIFIED Requirements

### Requirement: Config file location
The system SHALL read the model configuration from a JSON file. The default location SHALL be `ProjectDirs::config_dir("hi")/config.json`, and the system SHALL support an optional CLI-provided config file path override.

#### Scenario: Config file exists at default location
- **WHEN** the application starts without a config path override and `config.json` exists in the default config directory
- **THEN** the system SHALL parse the file and return a valid `ModelConfig`

#### Scenario: Config file does not exist at default location
- **WHEN** the application starts without a config path override and `config.json` does not exist in the default config directory
- **THEN** the system SHALL return an error with a message indicating the expected default config file path

#### Scenario: CLI override path exists
- **WHEN** the application starts with a shared CLI config path parameter and the specified file exists
- **THEN** the system SHALL load and validate `ModelConfig` from the specified path instead of the default location

#### Scenario: CLI override path does not exist
- **WHEN** the application starts with a shared CLI config path parameter and the specified file does not exist
- **THEN** the system SHALL return an error with a message indicating the specified config file path
