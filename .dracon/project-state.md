# Project State

## Current Focus
Version auto-increment is active (managed by build automation). All features wired and docs updated.

## Completed
- [x] Wired `generate_preview()` into batch processor export pipeline
- [x] Wired `BatchProgress` into batch/parallel batch processing for resume capability
- [x] Split `gui.rs` (2149 lines) into `gui.rs` + `gui/processing.rs` + `gui/tabs.rs`
- [x] Added `#[must_use]` to `Config::merge()`
- [x] Updated CHANGELOG with new 0.76.0 entry
- [x] Updated README with batch resume examples, `--preview`/`--notify` flags, config preview field
- [x] `cargo test --all-features` → 136 tests pass
- [x] `cargo clippy --all-features` → 0 warnings, 0 errors
