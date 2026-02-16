# Tasks: add-markdown-hierarchical-memory-tool

## Implementation

- [x] Create `package/hi-tools/src/memory.rs` with `MemoryTool` struct implementing `rig::tool::Tool`
- [x] Implement `parse_sections()` parser for hierarchical markdown headers (## / ### / ####)
- [x] Implement `parse_header_line()` helper for markdown header detection
- [x] Implement `read_memory()` — full file read, section read (with children), case-insensitive path matching
- [x] Implement `write_memory()` — create/update sections, auto-create parent headers for nested paths
- [x] Implement `list_sections()` — indented tree listing with full paths
- [x] Register `pub mod memory` and `pub use memory::MemoryTool` in `hi-tools/src/lib.rs`
- [x] Add `MemoryTool` to `build_tools()` in `hi-core/src/provider.rs` with `data_dir()/memory.md` path
- [x] Add `"memory: Read/write persistent hierarchical markdown memory"` to tool_descriptions in `send_message`
- [x] Add `"memory: Read/write persistent hierarchical markdown memory"` to tool_descriptions in `send_message_streaming`
- [x] Fix `parse_sections()` to skip content without headers (no empty-path sections)

## Tests (18 in `memory::tests`)

- [x] `test_parse_header_line` — header detection and level parsing
- [x] `test_parse_sections_empty` — empty input produces no sections
- [x] `test_parse_sections_no_headers` — headerless text produces no sections
- [x] `test_parse_sections_basic` — single header with content
- [x] `test_parse_sections_sibling_headers` — same-level headers
- [x] `test_parse_sections_back_to_parent_level` — nested then back to parent
- [x] `test_list_sections_empty` — empty file returns empty list
- [x] `test_list_sections_hierarchical` — nested section tree listing
- [x] `test_read_memory_full` — read entire memory file
- [x] `test_read_memory_section` — read specific section by path
- [x] `test_read_memory_section_includes_children` — parent read includes child sections
- [x] `test_read_memory_section_not_found` — missing section returns error
- [x] `test_read_memory_case_insensitive` — case-insensitive path matching
- [x] `test_read_memory_nonexistent` — nonexistent file returns empty
- [x] `test_write_memory_new_file` — write creates file with section
- [x] `test_write_memory_update_existing` — update replaces section content
- [x] `test_write_memory_nested_section` — nested path auto-creates parents
- [x] `test_write_memory_preserves_other_sections` — write doesn't clobber siblings
- [x] `test_write_memory_case_insensitive_match` — case-insensitive update

## Verification

- [x] `cargo check --workspace` — clean
- [x] `cargo test --workspace` — 155 passed, 0 failed
