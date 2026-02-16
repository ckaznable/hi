## ADDED Requirements

### Requirement: Send typing indicator before model inference
The Telegram adapter SHALL send a `ChatAction::Typing` action immediately upon receiving a non-command message, before session creation or model inference begins.

#### Scenario: User sends a chat message
- **WHEN** the bot receives a text message that is not a `/` command
- **THEN** the bot sends `ChatAction::Typing` to the originating `chat_id` before calling `session_manager.get_or_create()`

#### Scenario: User sends a slash command
- **WHEN** the bot receives a message starting with `/`
- **THEN** the bot SHALL NOT send a typing indicator (command responses are instant)

### Requirement: Maintain typing indicator during model response generation
The Telegram adapter SHALL periodically re-send `ChatAction::Typing` every 5 seconds while waiting for the model response, because Telegram expires the typing indicator after approximately 5 seconds.

#### Scenario: Model inference takes longer than 5 seconds
- **WHEN** model inference is in progress and 5 seconds have elapsed since the last typing action
- **THEN** the bot re-sends `ChatAction::Typing` to the same `chat_id`

#### Scenario: Model response completes
- **WHEN** the model response is fully received and ready to send
- **THEN** the periodic typing indicator task is aborted before the reply is sent

### Requirement: Typing indicator failures must not block message processing
The Telegram adapter SHALL treat typing indicator send failures as non-fatal. A failure to send `ChatAction::Typing` MUST NOT prevent the model response from being generated or delivered.

#### Scenario: Network error on typing action
- **WHEN** `send_chat_action(Typing)` fails due to a network error or Telegram API error
- **THEN** the error is logged as a warning and message processing continues normally

#### Scenario: Rate limit on typing action
- **WHEN** `send_chat_action(Typing)` returns a rate limit error
- **THEN** the periodic typing task logs the error and stops re-sending, but message processing continues normally
