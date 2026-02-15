## MODIFIED Requirements

### Requirement: User input
The system SHALL accept user text input via the TUI input field and send it to the LLM through hi-core.

#### Scenario: Send message
- **WHEN** the user types a message and presses Enter
- **THEN** the system SHALL send the message to hi-core, display a loading indicator, and show the assistant's response when received

#### Scenario: Empty input
- **WHEN** the user presses Enter with an empty input field
- **THEN** the system SHALL not send any message

#### Scenario: Stream assistant response updates
- **WHEN** streaming mode is enabled and the assistant response is produced in chunks
- **THEN** the TUI SHALL append and render assistant text incrementally during generation until completion

## ADDED Requirements

### Requirement: Streaming response accumulation
For each in-progress assistant response, the system SHALL accumulate streamed text chunks into one reused mutable `String` buffer instead of allocating a new string per chunk.

#### Scenario: Reuse single buffer for one response
- **WHEN** a streamed assistant response emits multiple text chunks
- **THEN** the system SHALL append each chunk into the same mutable `String` buffer using incremental append operations

#### Scenario: Persist final streamed response
- **WHEN** streaming completes successfully
- **THEN** the final accumulated buffer content SHALL be stored as the assistant message in chat history

#### Scenario: Fallback when streaming unavailable
- **WHEN** provider streaming is not available or streaming fails before completion
- **THEN** the system SHALL fall back to non-streaming response flow and still produce a complete assistant message in the TUI
