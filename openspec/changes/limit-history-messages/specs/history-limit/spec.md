## ADDED Requirements

### Requirement: History message limit
The system SHALL support limiting the number of conversation history messages sent to the LLM.

#### Scenario: History limit set
- **WHEN** config has `history_limit: 10`
- **THEN** the system SHALL send only the 10 most recent messages (excluding the current user message) to the LLM

#### Scenario: History limit not set
- **WHEN** config does not have `history_limit` field
- **THEN** the system SHALL send all available history messages (subject to context window limits)

#### Scenario: History limit exceeds available messages
- **WHEN** config has `history_limit: 100` but only 5 messages exist in history
- **THEN** the system SHALL send all 5 available messages (no error, acts as if unlimited)

#### Scenario: History limit set to zero
- **WHEN** config has `history_limit: 0`
- **THEN** the system SHALL send only the current user message (no prior history)

#### Scenario: History limit set to negative
- **WHEN** config has `history_limit: -1`
- **THEN** the system SHALL treat it as unlimited (same as field not present)
