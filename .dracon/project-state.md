# Project State

## Current Focus
All 4 big features implemented. Tests passing, clippy clean.

## Completed
- [x] **Task 1: GPU Hardware Acceleration**
  - `src/hwaccel.rs` — `HwAccel` enum with NVENC, AMF, VAAPI, VideoToolbox, auto-detection via `ffmpeg -hwaccels`
  - `FfmpegEditor` now stores `hw_accel`, swaps `libx264` for GPU codec in trim pipeline
  - CLI flag `--gpu <nvenc|amf|vaapi|videotoolbox|none|auto>`
  - Config field `video.hw_accel` with serde support
- [x] **Task 2: True Preview-Before-Render**
  - `export.preview_duration` config field (default 30s)
  - CLI flag `--preview-duration <SEC>`
  - Preview generated *before* heavy pipeline (after silence analysis, at ~11% progress)
- [x] **Task 3: Better Notifications & UI Feedback**
  - `indicatif` progress bars in `process_batch_dir` with ETA
  - Batch summary printed at end: successful / failed / skipped / total
  - GUI toast notifications (top-right, 5s auto-dismiss) for completed/failed files
  - Watch mode heartbeat shows last processed file
- [x] **Task 4: GUI Batch Queue + Process Now**
  - New `Queue` tab in GUI
  - `QueuedFile` struct with path, preset, status, progress
  - "+ Add Files" button (multi-select via `rfd::FileDialog`)
  - "Process All" button with processing state tracking
  - "Clear Completed" / "Clear All" buttons
  - Per-file status badge and progress bar
  - File removal from queue
- [x] `cargo test --all-features` → 142 tests pass (up from 136)
- [x] `cargo clippy --all-features` → 0 warnings, 0 errors
```
