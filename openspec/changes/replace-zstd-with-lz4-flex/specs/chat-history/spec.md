## MODIFIED Requirements

### Requirement: History storage format
The system SHALL store chat history as JSON serialized with LZ4 compression (lz4_flex frame format) at `ProjectDirs::data_dir("hi")/history.json.lz4`.

#### Scenario: Save history
- **WHEN** the chat session ends or a new message is added
- **THEN** the system SHALL serialize the message history to JSON and compress it with LZ4 frame format, writing to `history.json.lz4`

#### Scenario: Load history on startup
- **WHEN** the application starts and `history.json.lz4` exists in the data directory
- **THEN** the system SHALL decompress (LZ4 frame format) and deserialize the file to restore the previous chat history

#### Scenario: No existing history
- **WHEN** the application starts and `history.json.lz4` does not exist
- **THEN** the system SHALL initialize an empty history

### Requirement: Reset
The system SHALL provide a `reset` operation that clears all chat history.

#### Scenario: Reset history
- **WHEN** the user triggers a reset
- **THEN** the system SHALL clear all in-memory messages and delete the `history.json.lz4` file from disk

#### Scenario: Use after reset
- **WHEN** the user sends a message after reset
- **THEN** the system SHALL treat it as the start of a fresh conversation with no prior context
