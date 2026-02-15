## ADDED Requirements

### Requirement: Root CLI binary
The system SHALL provide a root CLI binary from a workspace member under `bin/` as the canonical entrypoint for command invocation.

#### Scenario: Workspace includes root CLI
- **WHEN** the workspace is resolved by Cargo
- **THEN** the root CLI crate under `bin/` SHALL be included in workspace members and build successfully

### Requirement: Clap-based subcommand routing
The root CLI SHALL parse commands using `clap` and dispatch behavior based on subcommands.

#### Scenario: Parse tui subcommand
- **WHEN** the user runs the root CLI with `tui`
- **THEN** the CLI SHALL parse the command successfully and route execution to TUI startup flow

#### Scenario: Invalid subcommand
- **WHEN** the user runs the root CLI with an unknown subcommand
- **THEN** the CLI SHALL return clap-generated usage/help output and a non-zero exit status

### Requirement: TUI launch integration
The root CLI `tui` subcommand SHALL start the same TUI application behavior currently provided by `hi-tui`.

#### Scenario: Launch TUI from root CLI
- **WHEN** the user runs the root CLI `tui` subcommand
- **THEN** the system SHALL load configuration, initialize terminal mode, and enter the interactive TUI chat loop

#### Scenario: Exit from TUI launched via root CLI
- **WHEN** the user exits the TUI launched through root CLI
- **THEN** the system SHALL restore terminal state and exit cleanly
