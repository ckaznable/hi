## 1. Skills description support

- [ ] 1.1 Update `Skill` struct to add `description: String`
- [ ] 1.2 Implement YAML frontmatter parsing: extract `description`; use filename when frontmatter is missing
- [ ] 1.3 Update `load_skills()` to parse frontmatter and separate body
- [ ] 1.4 Add unit tests for skill loading with and without frontmatter

## 2. ReadSkillsTool

- [ ] 2.1 Implement `ReadSkillsTool` in hi-tools with cached `Vec<SkillSummary>`
- [ ] 2.2 Implement rig `Tool` trait: no args, return `Vec<{ name, description }>`
- [ ] 2.3 Define `ToolDefinition` JSON schema
- [ ] 2.4 Update `builtin_tools()` to include `ReadSkillsTool`
- [ ] 2.5 Add unit tests for cases with skills and without skills

## 3. Context manager

- [ ] 3.1 Create `ContextManager` in hi-core with injected flag and preamble/tools/skills hashes
- [ ] 3.2 Implement `build_full_context_message()` to combine preamble + tool descriptions + skill summaries
- [ ] 3.3 Implement `build_delta_context_message()` to include only changed parts
- [ ] 3.4 Implement `should_inject()` based on injected flag and hash comparison
- [ ] 3.5 Implement `mark_dirty()` to set `injected = false` (used by compact/reset)
- [ ] 3.6 Implement `inject()` to prepend system message and update hashes
- [ ] 3.7 Add unit tests for first injection, no duplicate injection, reinjection after compact, and hash-change detection

## 4. Chat session integration

- [ ] 4.1 Refactor `ChatSession` to remove direct preamble injection and use `ContextManager`
- [ ] 4.2 Update `send_message()` to call `context_manager.inject()` before sending
- [ ] 4.3 Update `compact()` to call `context_manager.mark_dirty()` after completion
- [ ] 4.4 Update `reset()` to call `context_manager.mark_dirty()` after completion
- [ ] 4.5 Update agent construction to register `ReadSkillsTool` (5th tool)
- [ ] 4.6 Add integration test: first message includes context -> second does not -> includes again after compact
