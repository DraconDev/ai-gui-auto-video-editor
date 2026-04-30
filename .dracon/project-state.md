# Project State

## Current Focus
Add FFmpeg/FFprobe availability checks and refactor test assertions

## Completed
- [x] Added `has_ffmpeg()` and `has_ffprobe()` helper functions in `tests/common/mod.rs`.
- [x] Refactored chapter and captions assertions in `tests/gui_processing_tests.rs` to use boolean checks.
- [x] Updated `tests/pipeline_integration.rs` to remove early return after availability check.
- [x] Cargo.lock updated with new dependency versions.
