## Context

The workspace currently starts the UI by running `hi-tui` directly (`cargo run -p hi-tui --bin hi`). There is no root system CLI, no clap-based subcommand routing, and the TUI runtime is implemented inside `package/hi-tui/src/main.rs` as a binary-local flow.

This change introduces a top-level entrypoint crate under `bin/` and routes `tui` startup through it. Scope is extended with one additional requirement: support chat streaming output, with incremental accumulation using the same `String` buffer to reduce repeated allocations.

## Goals / Non-Goals

**Goals:**
- Add a workspace-level binary crate under `bin/` as the canonical system entrypoint.
- Add clap command parsing with an initial `tui` subcommand.
- Keep existing TUI behavior while enabling launch through the new root CLI.
- Add streaming response mode for chat flow, with chunk accumulation via a reused `String` buffer.

**Non-Goals:**
- No multi-session redesign.
- No web server/API surface.
- No provider-specific optimization beyond a generic streaming integration path.
- No broad UI redesign; only changes needed to support streaming updates.

## Decisions

1. Add a dedicated root CLI crate in `bin/hi` and include it in workspace members.
- Rationale: isolates command routing from feature crates and establishes a stable system entrypoint for future commands.
- Alternative considered: keep `hi-tui` as the only binary and add clap directly there. Rejected because it couples subcommand growth to UI crate concerns.

2. Keep `hi-tui` runtime reusable by extracting the TUI startup logic into callable code, then have both old binary path and new root CLI path reuse it.
- Rationale: avoids duplicating terminal setup/run loop logic and keeps behavior consistent.
- Alternative considered: root CLI spawning `hi-tui` as child process. Rejected because it complicates error propagation and control flow.

3. Add `tui` subcommand first, with minimal command surface.
- Rationale: delivers the requested entrypoint now while preserving room for future subcommands.
- Alternative considered: introduce multiple subcommands in first iteration. Rejected to keep blast radius small.

4. Implement streaming in the chat session/provider path using rig streaming APIs, and accumulate text deltas with a single mutable `String` (`push_str` on the same buffer).
- Rationale: matches rig streaming patterns and minimizes allocation churn during long responses.
- Alternative considered: concatenate per-chunk temporary strings. Rejected due to higher allocation/copy overhead.

5. Keep non-streaming as fallback mode.
- Rationale: preserves compatibility for providers or scenarios where streaming is unavailable or fails.
- Alternative considered: streaming-only behavior. Rejected to avoid regressions.

## Risks / Trade-offs

- [Runtime coupling between root CLI and TUI crate] → Mitigation: define a small reusable TUI launch API boundary and keep CLI orchestration thin.
- [Streaming behavior differences across providers] → Mitigation: provide a unified stream-to-buffer path and explicit fallback to existing non-streaming `chat` flow.
- [UI update frequency causing flicker or CPU overhead] → Mitigation: batch repaint cadence and only append changed assistant content per chunk.
- [Potential regressions in existing startup command] → Mitigation: keep backward-compatible launch path during migration and validate both paths.
