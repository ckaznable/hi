## ADDED Requirements

### Requirement: Root CLI uses argh for command parsing
The root `hi` CLI SHALL parse commands and arguments using `argh` while preserving existing command intents for `init`, `remote`, `config validate`, and `tui`.

#### Scenario: Parse supported top-level commands
- **WHEN** a user invokes `hi` with a supported command path (`init`, `remote`, `config validate`, or `tui`)
- **THEN** the CLI SHALL parse the input successfully and dispatch the corresponding command handler

#### Scenario: Reject unknown command path
- **WHEN** a user invokes `hi` with an unsupported command path
- **THEN** the CLI SHALL fail parsing and return non-zero exit behavior with parser-generated usage guidance

### Requirement: TUI support is opt-in for hi-cli builds
`hi-cli` SHALL treat TUI support as an explicit feature-gated capability, and default builds SHALL exclude TUI linkage.

#### Scenario: Build without TUI feature
- **WHEN** `hi-cli` is built without enabling the `tui` feature
- **THEN** the build output SHALL exclude `hi-tui` dependency linkage from the default feature set

#### Scenario: Build with TUI feature enabled
- **WHEN** `hi-cli` is built with the `tui` feature enabled
- **THEN** the build SHALL include `hi-tui` and allow the `tui` command path to start the TUI runtime

### Requirement: Disabled TUI invocation is actionable
When TUI support is not enabled in a build, invoking the `tui` path SHALL produce a clear, actionable failure response.

#### Scenario: User invokes tui on non-TUI build
- **WHEN** a user runs `hi tui` on a build where the `tui` feature is not enabled
- **THEN** the CLI SHALL exit non-zero and SHALL provide guidance to run/install with `--features tui`
