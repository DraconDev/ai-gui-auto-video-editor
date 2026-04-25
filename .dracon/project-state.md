# Project State

## Current Focus
Big review: UI/UX improvements, modularizing GUI, wiring dead modules, and syncing docs.

## Completed
- [x] Synced version numbers (Cargo.toml + CHANGELOG → 0.69.0)
- [x] Updated CHANGELOG with 0.69.0 entry (preview wiring, batch progress, GUI modularization)
- [x] Updated README with batch resume examples, `--preview` and `--notify` flags, config preview field
- [x] Wired `generate_preview()` into batch processor export pipeline (`--preview` now works end-to-end)
- [x] Wired `BatchProgress` into `process_batch_dir` and `process_batch_dir_parallel` for resume capability
- [x] Split `gui.rs` (2149 lines) into `gui.rs` + `gui/processing.rs` + `gui/tabs.rs`
- [x] Added `#[must_use]` to `Config::merge()`
- [x] `cargo test --all-features` → 136 tests pass
- [x] `cargo clippy --all-features` → 0 warnings, 0 errors
```
