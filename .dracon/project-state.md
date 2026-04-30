# ProjectState

## Current Focus
Release 13.2.0 – adds emoji font support, security hardening, comprehensive test expansion, and multiple bug‑fixes and new features.

## Completed- [x] Added desktop entry and SVG icon assets for the AI Video Editor application
- [x] Built and packaged the executable binary and release archives (tar.gz, sha256)
- [x] Updated documentation with changelog, README, customer‑facing guide, and release locations
- [x] Fixed GUI emoji rendering by configuring fallback emoji fonts (Noto Color Emoji, Segoe UI Emoji)
- [x] Implemented security fix for FFmpeg filter injection via `escape_ffmpeg_filter_path()`
- [x] Fixed caption‑burn data loss using atomic rename to preserve original video
- [x] Fixed Config::merge to retain non‑default values and not overwrite user settings
- [x] Fixed STT panic on short audio by guarding frame calculation
- [x] Fixed auto‑reframe to respect face movement across frames with temporal smoothing
- [x] Resolved overlapping silence segment creation and added deduplication
- [x] Added ffprobe availability check alongside ffmpeg at startup
- [x] Introduced RAII utilities `TempDir` and `TempFile` for safe temporary resources
- [x] Added comprehensive test suite (≈53 new tests) covering analyzer, batch processor, thumbnail, GUI, HW‑accel, ML, exporter, watermark, editor, utils, scene detection, presets, preview
- [x] Added preview export (`--preview` flag) and batch progress persistence with resume support
- [x] Modularized GUI codebase into `gui/processing.rs` and `gui/tabs.rs` for better maintainability
