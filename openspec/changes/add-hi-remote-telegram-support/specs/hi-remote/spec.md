## ADDED Requirements

### Requirement: Telegram inbound update intake
The system MUST support Telegram inbound updates for the `hi-remote` capability and process text messages through a polling-based update flow.

#### Scenario: Receive text message update
- **WHEN** Telegram returns an update containing a text message
- **THEN** the system MUST parse the message content and pass it into the remote chat handling pipeline

#### Scenario: Ignore unsupported update payload
- **WHEN** Telegram returns an update that does not contain a supported text message payload
- **THEN** the system MUST skip that update without terminating the remote runtime

### Requirement: Chat session ownership per Telegram chat
The system MUST maintain isolated chat session state per Telegram `chat_id` so messages from different chats do not share history or context.

#### Scenario: First message from chat
- **WHEN** a message arrives from a `chat_id` with no active session
- **THEN** the system MUST create a new chat session for that `chat_id`

#### Scenario: Follow-up message from same chat
- **WHEN** a subsequent message arrives from an existing `chat_id`
- **THEN** the system MUST route the message to the same chat session for continuity

### Requirement: Bridge Telegram messages to hi-core session flow
The system MUST use `hi-core` chat session APIs as the execution path for Telegram conversations instead of duplicating provider orchestration logic.

#### Scenario: Inbound message routed to core
- **WHEN** a Telegram text message is accepted for processing
- **THEN** the system MUST invoke the core chat session message API and obtain assistant output from that session flow

#### Scenario: Streaming-enabled response path
- **WHEN** the remote runtime uses streaming response mode
- **THEN** the system MUST consume streamed chunks and produce a final assistant reply for Telegram delivery

### Requirement: Telegram outbound reply delivery constraints
The system MUST deliver assistant responses back to Telegram and respect Telegram text message length constraints.

#### Scenario: Response within single-message limit
- **WHEN** the assistant reply length is within Telegram's allowed text limit
- **THEN** the system MUST send one Telegram message containing the assistant reply

#### Scenario: Response exceeds single-message limit
- **WHEN** the assistant reply length exceeds Telegram's single-message text limit
- **THEN** the system MUST split the reply into ordered message parts and send all parts to the same `chat_id`

### Requirement: Telegram API resilience and auth safety
The system MUST use explicit Telegram bot authentication configuration and handle recoverable API throttling failures.

#### Scenario: Missing bot token configuration
- **WHEN** Telegram runtime is enabled but bot token configuration is missing
- **THEN** the system MUST fail startup with a clear configuration error

#### Scenario: API rate-limit response
- **WHEN** Telegram API responds with a retry-after throttle signal
- **THEN** the system MUST delay retry according to the provided retry-after duration before reattempting the failed request
