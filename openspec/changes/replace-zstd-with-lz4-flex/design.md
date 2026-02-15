## Context

The `hi-history` crate compresses chat history using `zstd` (C bindings via `zstd-sys`). The compression is used in exactly two methods — `ChatHistory::load` (decompress) and `ChatHistory::save` (compress) — both in `package/hi-history/src/history.rs`. The storage format is `history.json.zst`.

Current zstd API usage:
- `zstd::Decoder::new(&compressed[..])` — streaming decompression from byte slice
- `decoder.read_to_end(&mut json_bytes)` — read all decompressed bytes
- `zstd::Encoder::new(Vec::new(), 3)` — streaming compression to Vec at level 3
- `encoder.write_all(&json)` — write all bytes
- `encoder.finish()` — finalize and return compressed Vec

The existing spec (`openspec/specs/chat-history/spec.md`) explicitly mentions zstd and `history.json.zst`.

## Goals / Non-Goals

**Goals:**
- Replace `zstd` with `lz4_flex` frame-format streaming API
- Change storage filename from `history.json.zst` to `history.json.lz4`
- Eliminate native C dependency (`zstd-sys`)
- Maintain identical load/save/reset behavior
- Update spec to reflect new compression algorithm

**Non-Goals:**
- Adding a compression abstraction layer or trait (overkill for a single call site)
- Supporting both formats simultaneously or providing migration tooling
- Changing the JSON serialization format
- Optimizing compression ratio (LZ4 is intentionally speed-over-ratio)

## Decisions

### Decision 1: Use lz4_flex frame format (not block format)

**Choice**: Use `lz4_flex::frame::FrameEncoder` / `FrameDecoder` which implement `std::io::Write` / `std::io::Read`.

**Rationale**: The current code uses zstd's streaming API (`Read`/`Write` traits). The lz4_flex frame format provides the same trait implementations, making it a near drop-in replacement. Block format would require buffering all data and tracking sizes manually.

**Alternative considered**: `lz4_flex::compress_prepend_size` / `decompress_size_prepended` (block API) — rejected because it changes the call pattern and doesn't support streaming, though for small chat histories it would also work.

### Decision 2: Direct swap without abstraction

**Choice**: Replace zstd calls inline in `load`/`save` rather than extracting a compression trait.

**Rationale**: There is exactly one call site for compression and one for decompression. An abstraction trait adds complexity with no practical benefit. If a third compression option is ever needed, extraction can happen then.

### Decision 3: No backward compatibility for existing history files

**Choice**: Existing `history.json.zst` files will simply not be found (the code looks for `history.json.lz4`), resulting in a fresh empty history.

**Rationale**: Chat history is ephemeral and non-critical. Users can reset. Adding dual-format detection increases complexity for a one-time migration that affects a personal tool.

**Alternative considered**: Auto-detect format by trying lz4 first, falling back to zstd — rejected because it requires keeping zstd as a dependency, defeating the purpose.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Existing users lose chat history on upgrade | Acceptable — history is ephemeral; document in changelog |
| LZ4 has ~5-10% worse compression ratio than zstd level 3 | Negligible for small JSON chat histories (typically < 100KB) |
| `lz4_flex` frame format has no dictionary support | Not needed — current code doesn't use zstd dictionaries |
| Orphaned `history.json.zst` files remain on disk | Users can manually delete; no auto-cleanup needed |

## Dependency Changes

| Before | After |
|--------|-------|
| `zstd = "0.13"` | `lz4_flex = "0.11"` |

> Note: Use the latest stable lz4_flex version. Only the default features are needed (frame format is included by default).
