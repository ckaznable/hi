## ADDED Requirements

### Requirement: Root CLI TUI entry behavior is feature-aware
The system SHALL keep TUI runtime behavior unchanged when TUI is compiled in, and SHALL provide explicit user guidance when TUI is not compiled into `hi-cli`.

#### Scenario: TUI runtime path when feature enabled
- **WHEN** a user runs `hi tui` on a build where the `tui` feature is enabled
- **THEN** the system SHALL start the same TUI runtime behavior provided by `hi_tui::run_tui()`

#### Scenario: TUI runtime path when feature disabled
- **WHEN** a user runs `hi tui` on a build where the `tui` feature is disabled
- **THEN** the system SHALL not attempt TUI initialization and SHALL return a clear remediation hint for enabling TUI support
