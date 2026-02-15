## Why

The `hi-history` crate uses the `zstd` crate (C bindings via `zstd-sys`) for compressing chat history to disk. Replacing it with `lz4_flex` — a pure-Rust LZ4 implementation — eliminates the native C dependency, simplifies cross-compilation, speeds up builds, and provides significantly faster decompression (~2-3×) at the cost of a slightly worse compression ratio (negligible for small JSON chat histories).

## What Changes

- **BREAKING**: Replace `zstd` dependency with `lz4_flex` in `hi-history`; existing `history.json.zst` files will no longer be readable — users must reset history or manually re-compress
- Change storage filename from `history.json.zst` to `history.json.lz4`
- Replace `zstd::Decoder` / `zstd::Encoder` streaming API with `lz4_flex::frame::FrameDecoder` / `lz4_flex::frame::FrameEncoder` in `ChatHistory::load` and `ChatHistory::save`
- Update all tests that reference `history.json.zst` to use `history.json.lz4`

## Capabilities

### New Capabilities

_(none — this is a dependency swap, not a feature addition)_

### Modified Capabilities

- `chat-history`: The compression algorithm changes from zstd to LZ4 (lz4_flex frame format), and the storage filename changes from `history.json.zst` to `history.json.lz4`. All behavioral requirements (save, load, reset, compact) remain identical.

## Impact

- **Code**: `package/hi-history/src/history.rs` (primary — `load` and `save` methods), `package/hi-history/Cargo.toml` (dependency swap)
- **Dependencies**: Remove `zstd = "0.13"` (C bindings + `zstd-sys`); add `lz4_flex` (pure Rust). Eliminates native build toolchain requirement.
- **Data**: Existing `history.json.zst` files become unreadable. This is acceptable — chat history is ephemeral and users can reset.
- **Specs**: `openspec/specs/chat-history/spec.md` needs a delta spec to update the compression algorithm and filename references.
- **Docs**: `README.md` mentions "zstd compression" and `history.json.zst` — needs updating.
- **Tests**: All roundtrip and reset tests in `history.rs` will be updated to use the new filename.
