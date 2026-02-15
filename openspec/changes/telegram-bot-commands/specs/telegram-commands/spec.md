## ADDED Requirements

### Requirement: /compact command triggers history compaction
The system SHALL compact the chat history when a user sends `/compact` command.

#### Scenario: Compact command succeeds
- **WHEN** user sends `/compact` to the bot
- **THEN** the chat history is compacted and confirmation message is sent

#### Scenario: Compact command with no history
- **WHEN** user sends `/compact` but there is no history to compact
- **THEN** a message indicating nothing to compact is sent

### Requirement: /new command resets conversation
The system SHALL clear the chat history when a user sends `/new` command.

#### Scenario: New command succeeds
- **WHEN** user sends `/new` to the bot
- **THEN** the chat history is cleared and confirmation message is sent

#### Scenario: New command on fresh session
- **WHEN** user sends `/new` but no session exists yet
- **THEN** a message indicating no conversation to reset is sent

### Requirement: /help command shows available commands
The system SHALL display help text when a user sends `/help` command.

#### Scenario: Help command
- **WHEN** user sends `/help` to the bot
- **THEN** a message listing available commands is sent

### Requirement: Unknown commands are handled gracefully
The system SHALL notify users when they send an unrecognized command.

#### Scenario: Unknown command
- **WHEN** user sends a command that is not recognized (e.g., `/unknown`)
- **THEN** a message suggesting to use `/help` is sent

### Requirement: Non-command messages are processed normally
The system SHALL treat regular text messages as chat input to the LLM.

#### Scenario: Regular message
- **WHEN** user sends a message that does not start with `/`
- **THEN** the message is processed as chat input to the LLM
