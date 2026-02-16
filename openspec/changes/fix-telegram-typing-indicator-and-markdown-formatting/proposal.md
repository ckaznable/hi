## Why

The Telegram bot sends replies as plain text (no `parse_mode` set), so markdown markers produced by the LLM appear as raw characters instead of formatted text. Additionally, the bot does not send a typing indicator before generating a response, leaving users with no feedback that their message is being processed — especially noticeable when model inference takes several seconds.

## What Changes

- Send `ChatAction::Typing` before starting model inference, with periodic re-sends every 5 seconds until the response is ready (Telegram typing indicator expires after ~5s).
- Set `parse_mode` to MarkdownV2 on outgoing messages so Telegram renders bold, italic, code blocks, and links.
- Add MarkdownV2 escaping for text outside formatting entities to prevent Telegram API parse errors.
- Add a fallback path: if a formatted send fails (parse error), retry as plain text so messages are never silently lost.
- Update `split_message` to be aware of code block boundaries so splitting does not break open formatting entities across chunks.

## Capabilities

### New Capabilities
- `telegram-typing-indicator`: Send typing action to Telegram before and during LLM response generation.
- `telegram-markdown-formatting`: Format outgoing Telegram messages with MarkdownV2 parse mode, escaping, and fallback.

### Modified Capabilities
_(none — no existing main specs are affected; the hi-remote spec from the original change is not in openspec/specs/)_

## Impact

- **Code**: `package/hi-remote/src/telegram.rs` — primary changes (typing indicator, parse mode, escaping, fallback, split awareness).
- **Dependencies**: No new crate dependencies; `teloxide` already exposes `ChatAction`, `ParseMode`, and `send_chat_action`.
- **APIs**: No public API changes; all changes are internal to the Telegram adapter.
- **Risk**: Low — changes are scoped to outbound message formatting and a non-critical UX indicator. Fallback ensures no message loss.
