## MODIFIED Requirements

### Requirement: Compact
The system SHALL provide a `compact` operation that reduces history size to stay within the model's context window.

#### Scenario: Auto-compact before context window limit
- **WHEN** the estimated token count of the history exceeds the configured compact trigger threshold (default 80% of `context_window`)
- **THEN** the system SHALL compact history using the configured strategy

#### Scenario: Compact preserves recent messages
- **WHEN** compact is triggered on a history with 20 messages using truncate strategy
- **THEN** the system SHALL retain the 10 most recent messages and discard the oldest 10

#### Scenario: Compact stores summary message
- **WHEN** compact is triggered using small-model strategy and summary generation succeeds
- **THEN** the system SHALL replace older messages with a single summary system message while retaining recent turns

#### Scenario: Compact fallback to truncate
- **WHEN** small-model compact fails for any reason
- **THEN** the system SHALL run truncation compact and keep the history in a valid persisted state
