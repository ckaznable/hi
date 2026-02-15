## ADDED Requirements

### Requirement: History storage format
The system SHALL store chat history as JSON serialized with zstd compression at `ProjectDirs::data_dir("hi")/history.json.zst`.

#### Scenario: Save history
- **WHEN** the chat session ends or a new message is added
- **THEN** the system SHALL serialize the message history to JSON and compress it with zstd, writing to `history.json.zst`

#### Scenario: Load history on startup
- **WHEN** the application starts and `history.json.zst` exists in the data directory
- **THEN** the system SHALL decompress and deserialize the file to restore the previous chat history

#### Scenario: No existing history
- **WHEN** the application starts and `history.json.zst` does not exist
- **THEN** the system SHALL initialize an empty history

### Requirement: Single session design
The system SHALL maintain exactly one chat session. All messages SHALL be appended to the same history.

#### Scenario: Resume conversation
- **WHEN** the user starts the application after a previous session
- **THEN** the system SHALL load the previous history and continue the conversation from where it left off

### Requirement: Reset
The system SHALL provide a `reset` operation that clears all chat history.

#### Scenario: Reset history
- **WHEN** the user triggers a reset
- **THEN** the system SHALL clear all in-memory messages and delete the `history.json.zst` file from disk

#### Scenario: Use after reset
- **WHEN** the user sends a message after reset
- **THEN** the system SHALL treat it as the start of a fresh conversation with no prior context

### Requirement: Compact
The system SHALL provide a `compact` operation that reduces history size to stay within the model's context window.

#### Scenario: Auto-compact before context window limit
- **WHEN** the estimated token count of the history exceeds 80% of the configured `context_window`
- **THEN** the system SHALL automatically remove the oldest messages, retaining the most recent 50% of messages

#### Scenario: Compact preserves recent messages
- **WHEN** compact is triggered on a history with 20 messages
- **THEN** the system SHALL retain the 10 most recent messages and discard the oldest 10

### Requirement: Token estimation
The system SHALL provide a token estimate for the current history using a simple heuristic of character count divided by 4.

#### Scenario: Estimate tokens
- **WHEN** the history contains messages totaling 4000 characters
- **THEN** the system SHALL estimate the token count as approximately 1000
