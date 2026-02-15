## ADDED Requirements

### Requirement: Context injection lifecycle
The system SHALL manage context injection via a `ContextManager` that tracks whether context has been injected and detects changes.

#### Scenario: First message injection
- **WHEN** the user sends the first message in a session (or after compact/reset)
- **THEN** the system SHALL inject a full system message containing the preamble, tool descriptions, and skill summaries at the beginning of the history

#### Scenario: Subsequent messages without changes
- **WHEN** the user sends a message after context has already been injected, and no skills/tools/preamble have changed
- **THEN** the system SHALL NOT re-inject any context

#### Scenario: Re-injection after compact
- **WHEN** compact is triggered on the history
- **THEN** the system SHALL mark context as not-injected, causing re-injection on the next message

#### Scenario: Re-injection after reset
- **WHEN** reset is triggered
- **THEN** the system SHALL mark context as not-injected, causing re-injection on the next message

### Requirement: Change detection
The system SHALL detect changes to preamble, tools, and skills by comparing hash values of their content.

#### Scenario: Skills changed
- **WHEN** the skills content hash differs from the last injected hash
- **THEN** the system SHALL inject a context update message describing the changes

#### Scenario: No changes detected
- **WHEN** all hashes match the last injected values
- **THEN** the system SHALL skip injection

### Requirement: Context message format
The full context system message SHALL contain sections for system prompt, available tools with descriptions, and available skills with names and descriptions.

#### Scenario: Full context message
- **WHEN** full context injection is triggered
- **THEN** the system SHALL create a system message with [System Prompt], [Available Tools], and [Available Skills] sections

#### Scenario: Delta context message
- **WHEN** a change is detected but context was previously injected
- **THEN** the system SHALL create a system message with only the changed sections prefixed by [Context Update]
