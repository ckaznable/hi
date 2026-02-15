## 1. Dependency Swap

- [x] 1.1 In `package/hi-history/Cargo.toml`, replace `zstd = "0.13"` with `lz4_flex = "0.11"` (or latest stable)
- [x] 1.2 Run `cargo check -p hi-history` to verify the dependency resolves and identify compilation errors

## 2. Update ChatHistory::load

- [x] 2.1 Change `history_path` from `data_dir.join("history.json.zst")` to `data_dir.join("history.json.lz4")`
- [x] 2.2 Replace `zstd::Decoder::new(&compressed[..])` with `lz4_flex::frame::FrameDecoder::new(&compressed[..])`
- [x] 2.3 Update error context strings from "zstd decoder" to "lz4 decoder"
- [x] 2.4 Keep `decoder.read_to_end(&mut json_bytes)` unchanged (FrameDecoder implements `Read`)

## 3. Update ChatHistory::save

- [x] 3.1 Replace `zstd::Encoder::new(Vec::new(), 3)` with `lz4_flex::frame::FrameEncoder::new(Vec::new())`
- [x] 3.2 Update error context strings from "zstd encoder" to "lz4 encoder"
- [x] 3.3 Keep `encoder.write_all(&json)` and `encoder.finish()` unchanged (FrameEncoder implements `Write` and has `finish()`)

## 4. Update Tests

- [x] 4.1 Update `test_reset` assertion from `"history.json.zst"` to `"history.json.lz4"`
- [x] 4.2 Verify `test_load_save_roundtrip` passes with new compression (no code changes needed — it uses `ChatHistory::load`/`save` which are already updated)

## 5. Update Documentation and Specs

- [x] 5.1 In `README.md`, replace all references to "zstd compression" with "LZ4 compression" and `history.json.zst` with `history.json.lz4`

## 6. Verify

- [x] 6.1 Run `cargo check -p hi-history` — zero errors
- [x] 6.2 Run `cargo test -p hi-history` — all tests pass
- [x] 6.3 Run `cargo check --workspace` — no workspace-wide breakage
- [x] 6.4 Run `cargo test --workspace` — all tests pass
