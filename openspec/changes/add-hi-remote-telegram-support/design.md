## Context

`hi` currently exposes an interactive terminal experience (`hi-tui`) and routes user input into `hi-core::session::ChatSession`. There is no runtime component that accepts external platform events or sends assistant replies back to third-party messaging APIs. This change introduces a dedicated remote adapter boundary so external channels can reuse the same `hi-core` conversation pipeline.

Initial scope is Telegram only. The design must preserve existing local TUI behavior, avoid coupling transport-specific logic into `hi-core`, and prepare the system to add additional remote channels later.

Constraints:
- Existing session flow is centered on `ChatSession` APIs and streaming replies through Tokio `mpsc` channels.
- Current history behavior is single-session oriented; remote traffic introduces multi-chat lifecycle needs.
- Telegram API usage requires secure bot token handling, retry-aware outbound calls, and message-size aware delivery.

## Goals / Non-Goals

**Goals:**
- Add a new `hi-remote` runtime boundary that bridges external messages into `ChatSession`.
- Support Telegram as the first remote channel with inbound message handling and outbound assistant replies.
- Define deterministic chat-session ownership for Telegram chats (one logical session per Telegram chat ID).
- Keep transport concerns (Telegram protocol, retries, formatting limits) outside `hi-core` business logic.
- Establish configuration shape for Telegram runtime settings without breaking existing config workflows.

**Non-Goals:**
- Supporting multiple remote platforms in this change (Slack/Discord/LINE are out of scope).
- Building a generic HTTP server framework for all providers.
- Redesigning `hi-core` provider/model architecture.
- Implementing advanced multi-tenant persistence or distributed state.

## Decisions

### 1) Add a dedicated workspace crate for remote adapters
We introduce a new workspace package `hi-remote` under `package/`.

Rationale:
- Matches existing workspace decomposition (`hi-core`, `hi-tui`, `hi-tools`).
- Keeps external transport integrations isolated from core agent logic.
- Provides a clear location for future adapters while limiting this change to Telegram.

Alternatives considered:
- Put Telegram logic directly in `hi-core`: rejected because it mixes transport concerns with conversation orchestration.
- Extend `hi-tui` to support remote mode: rejected because TUI and remote runtime have different lifecycle and deployment profiles.

### 2) Use an adapter architecture: Telegram event -> ChatSession -> Telegram reply
Inbound Telegram text messages are translated into `ChatSession` requests. Outbound assistant content is sent back through Telegram Bot API.

Rationale:
- Reuses existing context, tools, and provider flow from `ChatSession`.
- Minimizes duplicate orchestration logic.
- Aligns remote behavior with local TUI behavior.

Alternatives considered:
- Build a separate inference path bypassing `ChatSession`: rejected due to duplicated logic and divergence risk.

### 3) Session ownership model: one active session per Telegram chat ID
`hi-remote` maintains an in-process mapping from Telegram chat ID to a `ChatSession` instance (or session worker).

Rationale:
- Preserves conversational continuity per Telegram chat.
- Keeps concurrency control explicit and avoids interleaving messages from different chats.

Alternatives considered:
- Single global session for all chats: rejected because chat contexts would contaminate each other.
- Create a new session for every message: rejected because long-term memory and continuity are lost.

### 4) Start with long polling transport; defer webhook mode
Initial implementation uses Telegram long polling for operational simplicity.

Rationale:
- No public callback endpoint or TLS setup required.
- Faster local development and lower deployment friction for first release.

Alternatives considered:
- Webhook-first design: rejected for v1 due to extra infrastructure requirements and certificate/ingress complexity.

### 5) Stream handling policy: aggregate stream chunks before sending
`ChatSession::send_message_streaming` remains the core call path; stream chunks are buffered and emitted to Telegram with message-size aware splitting.

Rationale:
- Preserves compatibility with current core streaming behavior.
- Avoids excessive Telegram API call frequency from per-token updates.
- Reduces risk of rate-limit pressure while still producing timely responses.

Alternatives considered:
- Send every chunk as a Telegram edit/send: rejected due to noisy UX and higher rate-limit risk.
- Disable streaming path and use non-streaming API only: rejected because it diverges from current interaction model and removes flexibility.

### 6) Configuration extension under existing config workflow
Remote settings are added as optional configuration entries (Telegram bot token, polling options, runtime toggles) while keeping current model/provider config unchanged.

Rationale:
- Preserves backward compatibility for existing users.
- Keeps remote runtime configurable without requiring a second config source.

Alternatives considered:
- Separate config file just for remote runtime: rejected for initial rollout to avoid split configuration UX.

## Risks / Trade-offs

- [Per-chat state growth in long-running process] -> Mitigation: add inactivity timeout and bounded cache strategy for idle sessions.
- [Telegram API rate limits during heavy usage] -> Mitigation: batch/aggregate output, apply retry with backoff, and cap send frequency per chat.
- [Single-process crash loses in-memory session map] -> Mitigation: rely on persisted history and reconstruct sessions lazily when chats resume.
- [Long polling operational lag vs webhook] -> Mitigation: tune poll timeout/interval and keep webhook support as a follow-up capability.
- [Config expansion may increase validation complexity] -> Mitigation: mark remote config as optional and enforce channel-specific validation only when enabled.

## Migration Plan

1. Add `hi-remote` crate to workspace without changing existing CLI/TUI entrypoints.
2. Implement Telegram adapter runtime behind explicit startup command/path so behavior is opt-in.
3. Introduce optional Telegram configuration fields with backward-compatible defaults.
4. Validate coexistence: TUI-only workflows continue to work unchanged when remote runtime is not enabled.
5. Rollback strategy: disable remote runtime configuration or stop running `hi-remote` process; core and TUI remain unaffected.

## Open Questions

- Should Telegram chat session persistence use shared history layout or a dedicated per-chat history path?
- What is the exact outbound formatting policy for long responses (split size, Markdown mode, and fallback behavior)?
- Do we need a configurable allowlist for permitted Telegram chat IDs in v1?
- Should message deduplication logic be included now to guard against update replay, or deferred to a follow-up hardening change?
