## Context

The Telegram adapter in `package/hi-remote/src/telegram.rs` uses teloxide to poll for updates and send replies. Currently:

1. **No typing indicator**: After receiving a message, the bot silently processes it through `session.send_message_streaming()` which may take 5–30+ seconds. The user sees no feedback until the final reply arrives.

2. **No parse mode**: `bot.send_message(ChatId(chat_id), text)` is called without `.parse_mode(...)`, so Telegram treats all outgoing messages as plain text. LLM-generated markdown markers (`*bold*`, `` `code` ``, etc.) appear as literal characters.

3. **No escaping or fallback**: Since no parse mode is set, there is no escaping logic and no fallback path for malformed formatting.

Key constraints:
- Telegram's `sendChatAction` typing indicator expires after ~5 seconds server-side.
- MarkdownV2 requires escaping 18+ special characters outside formatting entities.
- LLM output is unpredictable — may contain arbitrary markdown, partial entities, or characters that break MarkdownV2 parsing.
- Messages are split at 4096-char boundaries by `split_message`, which is not formatting-aware.

## Goals / Non-Goals

**Goals:**
- Show "typing…" indicator immediately when a message is received and maintain it until the reply is sent.
- Send outgoing messages with MarkdownV2 formatting so Telegram renders bold, italic, code blocks, and links.
- Escape text correctly for MarkdownV2 outside of formatting entities.
- Fall back to plain text if Telegram rejects the formatted message (parse error), ensuring no message is silently lost.

**Non-Goals:**
- Converting LLM markdown to Telegram-specific MarkdownV2 AST (too complex; we escape and send as-is, letting Telegram parse what it can).
- HTML parse mode (MarkdownV2 is closer to what LLMs naturally produce).
- Formatting-aware message splitting (complex; deferred — plain text fallback handles split-broken entities).

## Decisions

### 1. Typing indicator: spawn a periodic background task

**Choice**: In `handle_message`, immediately after entering the function (before `session_manager.get_or_create`), send an initial `ChatAction::Typing` and spawn a `tokio::spawn` task that re-sends it every 5 seconds. Abort the task when the model response is ready.

**Why**: Session creation and model inference can take 5–30+ seconds. Sending typing before `get_or_create` ensures the indicator appears even during slow session init. The periodic resend covers the full generation window since Telegram expires the indicator after ~5s.

**Alternative considered**: Single typing action at start — rejected because it would expire before most model responses complete.

**Alternative considered**: Cross-crate event/notifier from hi-core — rejected as over-engineered; all typing logic stays in hi-remote with no API changes to hi-core.

### 2. MarkdownV2 parse mode with escape-and-fallback

**Choice**: Add `.parse_mode(ParseMode::MarkdownV2)` to `send_message` calls. Before sending, escape MarkdownV2 special characters in text that is outside code blocks (``` ``` ```) and inline code (`` ` ``). If Telegram returns a parse error (400 Bad Request), retry without parse mode (plain text).

**Why**: LLM output naturally contains markdown that MarkdownV2 can render. Escaping only outside code blocks preserves code formatting. The fallback ensures reliability — a failed format never loses the message.

**Alternative considered**: HTML parse mode — rejected because LLM output is closer to markdown than HTML. Converting markdown→HTML would require a parser.

**Alternative considered**: Strip all markdown before sending — rejected as it removes useful formatting that Telegram can render.

### 3. Escaping strategy: protect code blocks, escape everything else

**Choice**: Implement `escape_markdown_v2(text)` that:
1. Identifies code block regions (triple backtick fences) and inline code (single backtick).
2. Leaves content inside code regions untouched.
3. Escapes all MarkdownV2 special characters (`_`, `*`, `[`, `]`, `(`, `)`, `~`, `` ` ``, `>`, `#`, `+`, `-`, `=`, `|`, `{`, `}`, `.`, `!`, `\`) in non-code text.

**Why**: Code blocks in MarkdownV2 must NOT have their content escaped (Telegram handles them literally). Escaping everything else prevents parse errors from stray special characters in natural language text.

### 4. Fallback: catch API error and retry plain

**Choice**: In `send_message_with_retry`, first attempt with MarkdownV2. If the error is a Telegram API error (not a rate limit), retry once without parse mode. Log the fallback as a warning.

**Why**: LLM output is unpredictable. Even with escaping, edge cases may produce invalid MarkdownV2. A plain-text fallback guarantees message delivery. The warning log enables debugging without user impact.

### 5. Typing indicator for commands: skip

**Choice**: Do not send typing indicator for `/` commands (handled by `handle_command`).

**Why**: Command responses are computed locally (no model call) and return near-instantly. Typing indicator would flash unnecessarily.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| MarkdownV2 escaping may over-escape, producing `\*` literals where bold was intended | Accept: LLM output is unpredictable; over-escaping is safer than parse errors. Users still get readable text. |
| Splitting messages may break formatting entities across chunks | Fallback to plain text handles this: if chunk 1 has an open `*` with no close, Telegram rejects it, fallback sends plain. |
| Typing action `send_chat_action` failures on network issues | Log warning, continue — typing is UX sugar and must not abort message processing. |
| Rate limiting on `send_chat_action` (counts toward Telegram limits) | Send at 5s intervals (well within per-chat limits). If rate-limited, the periodic task logs and stops. |
| Edge case: LLM produces MarkdownV2 that looks valid but renders wrong | Accept: same risk as any markdown renderer. Plain text fallback is the safety net. |
