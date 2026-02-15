## MODIFIED Requirements

### Requirement: History integration
The system SHALL integrate with the `chat-history` module for persistence, compact, and reset. After compact or reset, the `ContextManager` SHALL be notified to re-inject context on the next message.

#### Scenario: Auto-compact before sending
- **WHEN** the user sends a message and the token estimate exceeds the compact trigger threshold
- **THEN** the system SHALL run compact on the history before sending the message to the LLM

#### Scenario: Auto-compact with small model
- **WHEN** compact strategy is configured as `small-model` and compact is triggered
- **THEN** the system SHALL resolve the configured compact model (default `small`) and generate a summary used to compact history

#### Scenario: Auto-compact records current user language
- **WHEN** compact strategy is `small-model` and compact is triggered
- **THEN** the session SHALL include the current user language marker in the compacted context before sending subsequent prompts

#### Scenario: Auto-compact fallback on small-model error
- **WHEN** compact strategy is `small-model` and compact model invocation fails
- **THEN** the system SHALL fallback to truncation compact and continue processing the user message

#### Scenario: Auto-compact triggers re-injection
- **WHEN** compact is triggered
- **THEN** the system SHALL call `context_manager.mark_dirty()` so context is re-injected on the next message

#### Scenario: Reset via core
- **WHEN** a reset is triggered through the core
- **THEN** the system SHALL call `history.reset()` and clear the in-memory state

#### Scenario: Reset triggers re-injection
- **WHEN** reset is triggered
- **THEN** the system SHALL call `context_manager.mark_dirty()` so context is re-injected on the next message
