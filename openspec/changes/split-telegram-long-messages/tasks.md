## 1. UTF-8 Character Boundary Safety

- [x] 1.1 Replace byte-length comparisons in split_message() with char-count comparisons
- [x] 1.2 Use char_indices() for finding safe split points within MAX_MESSAGE_LENGTH chars
- [x] 1.3 Preserve existing newline-aware chunking logic

## 2. Testing

- [x] 2.1 Verify all 6 existing tests still pass
- [x] 2.2 Add multi-byte UTF-8 test (CJK + emoji) confirming no mid-character splits
- [x] 2.3 Run cargo test -p hi-remote — all 16 tests pass
