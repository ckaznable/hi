## ADDED Requirements

### Requirement: Provider client creation
The system SHALL create a rig provider client based on the `ModelConfig` provider field.

#### Scenario: Create OpenAI client
- **WHEN** the config specifies `provider: "openai"` with a valid `api_key`
- **THEN** the system SHALL create an `openai::Client` and build an agent with the configured model and preamble

#### Scenario: Create Anthropic client
- **WHEN** the config specifies `provider: "anthropic"` with a valid `api_key`
- **THEN** the system SHALL create an `anthropic::Client` and build an agent with the configured model and preamble

#### Scenario: Create Gemini client
- **WHEN** the config specifies `provider: "gemini"` with a valid `api_key`
- **THEN** the system SHALL create a `gemini::Client` and build an agent with the configured model and preamble

#### Scenario: Create Ollama client
- **WHEN** the config specifies `provider: "ollama"`
- **THEN** the system SHALL create an `ollama::Client` (no API key required) and build an agent with the configured model and preamble

### Requirement: Multi-turn chat
The system SHALL support multi-turn conversations by passing the full message history to the rig `Chat` trait on each prompt.

#### Scenario: Send message with history
- **WHEN** the user sends a message and the history contains previous messages
- **THEN** the system SHALL call `agent.chat()` with the new message and the existing history, and append both the user message and the assistant response to the history

#### Scenario: First message
- **WHEN** the user sends the first message with empty history
- **THEN** the system SHALL call `agent.chat()` with the message and an empty history vector

### Requirement: Built-in tools integration
The system SHALL register all built-in tools (`BashTool`, `ListFilesTool`, `ReadFileTool`, `WriteFileTool`, `ReadSkillsTool`) with the rig agent at build time.

#### Scenario: Agent with tools
- **WHEN** the agent is built
- **THEN** all five built-in tools SHALL be registered via `.tool()` and available for LLM function calling

### Requirement: Skills integration
The system SHALL use the `ContextManager` to inject skill information. Skills SHALL NOT be injected directly into the preamble every time. Instead, skill summaries (names + descriptions) SHALL be included in the context system message managed by `ContextManager`.

#### Scenario: Agent with skills via context manager
- **WHEN** skills are loaded and the first message is sent
- **THEN** the ContextManager SHALL inject a system message containing the preamble, tool descriptions, and skill summaries

#### Scenario: Skills not re-injected on subsequent messages
- **WHEN** skills have not changed and context was already injected
- **THEN** the system SHALL NOT re-inject skill information

### Requirement: History integration
The system SHALL integrate with the `chat-history` module for persistence, compact, and reset. After compact or reset, the `ContextManager` SHALL be notified to re-inject context on the next message.

#### Scenario: Auto-compact before sending
- **WHEN** the user sends a message and the token estimate exceeds 80% of `context_window`
- **THEN** the system SHALL run compact on the history before sending the message to the LLM

#### Scenario: Auto-compact triggers re-injection
- **WHEN** compact is triggered
- **THEN** the system SHALL call `context_manager.mark_dirty()` so context is re-injected on the next message

#### Scenario: Reset via core
- **WHEN** a reset is triggered through the core
- **THEN** the system SHALL call `history.reset()` and clear the in-memory state

#### Scenario: Reset triggers re-injection
- **WHEN** reset is triggered
- **THEN** the system SHALL call `context_manager.mark_dirty()` so context is re-injected on the next message

### Requirement: Preamble configuration
The system SHALL configure the rig agent's preamble (system prompt) from the `ModelConfig.preamble` field. If not provided, no base preamble SHALL be set (skills may still be appended).

#### Scenario: Custom preamble
- **WHEN** the config contains `preamble: "You are a helpful assistant."`
- **THEN** the agent SHALL use this as the base system prompt

#### Scenario: No preamble
- **WHEN** the config does not contain a `preamble` field
- **THEN** the agent SHALL be built without a base system prompt (skills-only context if any)
