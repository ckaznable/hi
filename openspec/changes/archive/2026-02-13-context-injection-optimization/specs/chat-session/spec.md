## MODIFIED Requirements

### Requirement: Skills integration
The system SHALL use the `ContextManager` to inject skill information. Skills SHALL NOT be injected directly into the preamble every time. Instead, skill summaries (names + descriptions) SHALL be included in the context system message managed by `ContextManager`.

#### Scenario: Agent with skills via context manager
- **WHEN** skills are loaded and the first message is sent
- **THEN** the ContextManager SHALL inject a system message containing the preamble, tool descriptions, and skill summaries

#### Scenario: Skills not re-injected on subsequent messages
- **WHEN** skills have not changed and context was already injected
- **THEN** the system SHALL NOT re-inject skill information

### Requirement: Built-in tools integration
The system SHALL register all built-in tools (`BashTool`, `ListFilesTool`, `ReadFileTool`, `WriteFileTool`, `ReadSkillsTool`) with the rig agent at build time.

#### Scenario: Agent with tools
- **WHEN** the agent is built
- **THEN** all five built-in tools SHALL be registered via `.tool()` and available for LLM function calling

### Requirement: History integration
The system SHALL integrate with the `chat-history` module for persistence, compact, and reset. After compact or reset, the `ContextManager` SHALL be notified to re-inject context on the next message.

#### Scenario: Auto-compact triggers re-injection
- **WHEN** compact is triggered
- **THEN** the system SHALL call `context_manager.mark_dirty()` so context is re-injected on the next message

#### Scenario: Reset triggers re-injection
- **WHEN** reset is triggered
- **THEN** the system SHALL call `context_manager.mark_dirty()` so context is re-injected on the next message
