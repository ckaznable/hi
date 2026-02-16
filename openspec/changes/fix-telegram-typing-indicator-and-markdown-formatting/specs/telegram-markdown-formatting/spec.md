## ADDED Requirements

### Requirement: Send messages with MarkdownV2 parse mode
The Telegram adapter SHALL set `parse_mode` to `MarkdownV2` on all outgoing chat reply messages so that Telegram renders supported markdown formatting (bold, italic, code, links).

#### Scenario: Normal message send
- **WHEN** the bot sends a reply message to a chat
- **THEN** the message is sent with `parse_mode=MarkdownV2`

### Requirement: Escape MarkdownV2 special characters outside code blocks
The Telegram adapter SHALL escape all MarkdownV2 special characters (`_`, `*`, `[`, `]`, `(`, `)`, `~`, `` ` ``, `>`, `#`, `+`, `-`, `=`, `|`, `{`, `}`, `.`, `!`, `\`) in text that is outside code block fences (triple backtick) and inline code (single backtick) regions.

#### Scenario: Text with special characters outside code
- **WHEN** the message text contains `Hello! How are you?` (no code blocks)
- **THEN** the `!` and `?` are escaped as `\!` and `\?` before sending (note: `?` is not a special char so only `!` and `.` type chars are escaped)

#### Scenario: Text inside a code block
- **WHEN** the message text contains a fenced code block (`` ``` ... ``` ``)
- **THEN** the content inside the code block is NOT escaped

#### Scenario: Text inside inline code
- **WHEN** the message text contains inline code (`` `code here` ``)
- **THEN** the content inside the inline code backticks is NOT escaped

#### Scenario: Mixed code and non-code text
- **WHEN** the message contains both code blocks and regular text
- **THEN** only the regular text portions are escaped; code block contents remain untouched

### Requirement: Fall back to plain text on formatting errors
The Telegram adapter SHALL retry a message send without `parse_mode` (plain text) if Telegram rejects the MarkdownV2-formatted message with a parse error. The fallback MUST log a warning.

#### Scenario: Telegram rejects MarkdownV2 formatted message
- **WHEN** `send_message` with `parse_mode=MarkdownV2` returns a Telegram API error (non-rate-limit, e.g. 400 Bad Request)
- **THEN** the bot retries sending the same text without `parse_mode` (plain text) and logs a warning about the fallback

#### Scenario: Telegram rejects with rate limit
- **WHEN** `send_message` returns a 429 rate limit error
- **THEN** the existing retry-after logic handles it (no fallback to plain text; rate limits are not parse errors)

#### Scenario: Plain text fallback also fails
- **WHEN** the plain text fallback send also fails
- **THEN** the error is handled by the existing retry logic in `send_message_with_retry`
