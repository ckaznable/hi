## ADDED Requirements

### Requirement: TUI chat interface
The system SHALL provide a terminal-based chat interface using ratatui that displays a scrollable message history and an input area.

#### Scenario: Display chat messages
- **WHEN** the chat history contains messages
- **THEN** the TUI SHALL display user messages and assistant responses in a scrollable area with visual distinction between sender types

#### Scenario: Empty state
- **WHEN** the application starts with no history
- **THEN** the TUI SHALL display an empty chat area with the input field ready for typing

### Requirement: User input
The system SHALL accept user text input via the TUI input field and send it to the LLM through hi-core.

#### Scenario: Send message
- **WHEN** the user types a message and presses Enter
- **THEN** the system SHALL send the message to hi-core, display a loading indicator, and show the assistant's response when received

#### Scenario: Empty input
- **WHEN** the user presses Enter with an empty input field
- **THEN** the system SHALL not send any message

### Requirement: Async LLM communication
The system SHALL perform LLM requests asynchronously so the TUI remains responsive during API calls.

#### Scenario: Non-blocking UI during response
- **WHEN** the system is waiting for an LLM response
- **THEN** the TUI SHALL remain responsive to user input and display events (e.g. scrolling, resizing)

### Requirement: Reset command
The system SHALL provide a way for the user to reset the chat history from the TUI.

#### Scenario: User triggers reset
- **WHEN** the user invokes the reset action
- **THEN** the system SHALL call `reset` on hi-core, clear the displayed messages, and show a fresh chat area

### Requirement: Application exit
The system SHALL allow the user to exit the application gracefully.

#### Scenario: Exit application
- **WHEN** the user presses the quit key binding
- **THEN** the system SHALL save the current history and restore the terminal to its original state before exiting
