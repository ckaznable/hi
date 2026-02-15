## ADDED Requirements

### Requirement: Small-model compaction strategy
The system SHALL support an LLM-based compaction strategy that summarizes older conversation history using a configured compact model (defaulting to `small`) before sending the next user message.

#### Scenario: Trigger small-model compact
- **WHEN** compact is enabled, strategy is `small-model`, and history token estimate exceeds the configured trigger threshold
- **THEN** the system SHALL invoke the configured compact model with a summarization prompt and compact the history using the model output

### Requirement: Summary output integration
The system SHALL integrate compact output as a summary system message while preserving recent conversation turns for continuity.

#### Scenario: Preserve recent turns after summary compact
- **WHEN** LLM compact succeeds
- **THEN** the system SHALL keep recent messages unchanged and replace older messages with a single summary system message

### Requirement: Language continuity marker in compact context
The system SHALL include the user's current language as an explicit marker in the compacted context so subsequent responses continue in the same language.

#### Scenario: Compact includes user language marker
- **WHEN** compact summary is generated and the current interaction language can be determined
- **THEN** the compacted context SHALL include a clear language marker indicating the user's current language

#### Scenario: Assistant follows marked language after compact
- **WHEN** the next response is generated after compaction
- **THEN** the assistant SHALL prioritize the language marker in compacted context and reply in that language unless the user explicitly switches language

### Requirement: Fallback safety
The system SHALL fall back to truncation compact when LLM compact cannot complete successfully.

#### Scenario: Compact model failure
- **WHEN** compact model resolution, request, or response parsing fails
- **THEN** the system SHALL execute truncation compact and continue normal chat flow without failing the user request
