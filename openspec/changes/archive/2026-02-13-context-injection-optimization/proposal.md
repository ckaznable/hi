## Why

In the current design, system prompt text, tool descriptions, and skill content are sent in full on every chat turn, which wastes tokens and adds latency. We need to optimize context injection so full context is injected only on the first turn, and reinjected only after compact/reset/content changes. We also need a built-in tool to read skills so the LLM can actively query skill details.

## What Changes

- Add built-in `ReadSkillsTool` so the LLM can query available skill names and descriptions
- Make system prompt (`preamble`) lazy-injected: inject only on the first message into history
- Inject tool descriptions and skill summaries in the first system message only
- Reinject full context after compact or reset
- Inject only the delta when skills/tools change
- Add `description` field to skills for `ReadSkillsTool` listing

## Capabilities

### New Capabilities

- `context-manager`: manages context-injection lifecycle and decides when to inject/reinject preamble, tool info, and skill summaries

### Modified Capabilities

- `builtin-tools`: add `ReadSkillsTool`
- `skills-loader`: add `description` to skills and provide summaries for tool listing
- `chat-session`: use context manager instead of always sending full preamble

## Impact

- **hi-core**: add context-manager module
- **hi-tools**: add `ReadSkillsTool`
- **shared**: add `description` field to skill structure
- **chat-session**: refactor injection logic
