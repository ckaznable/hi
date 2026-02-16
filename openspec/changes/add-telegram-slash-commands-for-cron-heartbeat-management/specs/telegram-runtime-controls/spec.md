## ADDED Requirements

### Requirement: Cron commands are handled directly by program logic
The system SHALL provide Telegram slash commands to view and edit cron settings without invoking LLM inference.

#### Scenario: View cron configuration
- **WHEN** user sends a valid cron view command
- **THEN** the system SHALL return the current cron configuration directly from program state or persisted config

#### Scenario: Edit cron configuration
- **WHEN** user sends a valid cron edit command with required parameters
- **THEN** the system SHALL validate and persist the cron update directly, then return a success confirmation

### Requirement: Heartbeat commands are handled directly by program logic
The system SHALL provide Telegram slash commands to view and edit heartbeat settings without invoking LLM inference.

#### Scenario: View heartbeat configuration
- **WHEN** user sends a valid heartbeat view command
- **THEN** the system SHALL return current heartbeat status and key fields (enabled state and interval)

#### Scenario: Edit heartbeat configuration
- **WHEN** user sends a valid heartbeat edit command with required parameters
- **THEN** the system SHALL validate and apply the heartbeat update directly, then return a success confirmation

### Requirement: MCP status and toggle commands are supported
The system SHALL support Telegram slash commands to inspect and toggle MCP state directly.

#### Scenario: Query MCP status
- **WHEN** user sends `/mcp`
- **THEN** the system SHALL return whether MCP is currently enabled or disabled

#### Scenario: Enable MCP
- **WHEN** user sends `/mcp on`
- **THEN** the system SHALL set MCP to enabled and return a confirmation message

#### Scenario: Disable MCP
- **WHEN** user sends `/mcp off`
- **THEN** the system SHALL set MCP to disabled and return a confirmation message

### Requirement: Invalid command format returns usage example
The system SHALL reject invalid slash command formats and return a concrete example message immediately.

#### Scenario: Invalid MCP argument
- **WHEN** user sends `/mcp enable`
- **THEN** the system SHALL reject the input and return `Usage: /mcp <on|off>`

#### Scenario: Invalid heartbeat command format
- **WHEN** user sends a heartbeat command missing required arguments
- **THEN** the system SHALL reject the input and return a heartbeat command usage example

#### Scenario: Invalid cron command format
- **WHEN** user sends a cron command with malformed or missing required arguments
- **THEN** the system SHALL reject the input and return a cron command usage example

### Requirement: Slash command actions do not trigger LLM processing
Operational slash commands for cron, heartbeat, and MCP SHALL be handled entirely in program control flow.

#### Scenario: Command short-circuit
- **WHEN** user sends any supported operational slash command
- **THEN** the system SHALL execute the command handler and return a response without creating an LLM request
