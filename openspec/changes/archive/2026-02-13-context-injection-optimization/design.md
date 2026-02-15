## Context

Based on the archived `simple-llm-chat` design, the system already has a preamble + skills injection mechanism. It currently sends full context every turn and should be optimized to lazy injection.

## Goals / Non-Goals

**Goals:**

- Add a context manager to control injection timing
- Add `ReadSkillsTool` so the LLM can query skills
- Inject full context on first message (`preamble + tool descriptions + skill summaries`)
- Reinject after compact/reset
- Inject only deltas when skills/tools change

**Non-Goals:**

- No token-level diffing
- No dynamic runtime skill reloading (still requires restart)

## Decisions

### 1. Context manager structure

```rust
pub struct ContextManager {
    injected: bool,              // whether context has already been injected
    last_preamble_hash: u64,     // hash of preamble at last injection
    last_tools_hash: u64,        // hash of tool descriptions at last injection
    last_skills_hash: u64,       // hash of skills at last injection
}
```

**Injection decision rules:**
- `injected == false` -> inject full context (first run or after compact/reset)
- `injected == true` and any hash mismatch -> inject only changed parts
- `injected == true` and all hashes match -> do not inject

**Injection method:** prepend context as a system message (`Message::system()`) in history.

**Rationale:** hash-based checks avoid unnecessary string comparisons. System-message injection remains compatible with rig `Chat` trait.

### 2. Context message format

On first injection, create one full system message:

```
[System Prompt]
{preamble content}

[Available Tools]
- bash: Execute bash commands
- list_files: List directory contents
- read_file: Read file content
- write_file: Write file content
- read_skills: List available skills with descriptions

[Available Skills]
- coding-assistant: Expert coding guidance and best practices
- translator: Multi-language translation assistant
```

On change, inject delta-only message:

```
[Context Update]
Skills changed:
- ADDED: new-skill: Description of new skill
Tools changed:
- ADDED: new_tool: Description of new tool
```

### 3. ReadSkillsTool

```rust
pub struct ReadSkillsTool {
    skills: Vec<SkillSummary>,  // cached at startup
}

// Args: none (list all skills)
// Output: Vec<{ name, description }>
```

When LLM calls this tool, it returns names and descriptions for all loaded skills.

### 4. Skill `description` field

Skill markdown uses YAML frontmatter for description:

```markdown
---
description: Expert coding guidance and best practices
---

You are a coding expert...
```

If frontmatter is missing, use filename as description.

### 5. Compact/Reset and context reinjection

- After `compact()`, set `injected = false`; reinject on next `send_message()`
- After `reset()`, set `injected = false`; same behavior
- Replace stored hashes with current values on reinjection for consistency

### 6. Dependency changes

| Crate | Package | Purpose |
|-------|---------|------|
| None | — | No new dependency needed; use std hash |

## Risks / Trade-offs

- **System-message behavior**: providers may handle system messages differently -> use rig `Message` abstraction for a unified path
- **Hash collision**: extremely low probability and acceptable
- **Context token footprint**: first system message consumes tokens, but only once, so total usage is lower
