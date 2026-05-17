# AI Video Editor - Agent Reference

## Project Overview

AI-powered video editor for content creators. Processes raw footage through automated pipelines: silence detection/cutting, audio enhancement, scene-change detection, auto-reframe, and more. Supports both CLI and GUI modes from a single binary.

## Repository Structure

```
ai-vid-editor/
├── src/
│   ├── main.rs              # CLI entry point, arg parsing, watch mode, signal handling
│   ├── lib.rs               # Library exports (incl. pub tests_common)
│   ├── analyzer.rs          # Silence detection, scene detection
│   ├── batch_processor.rs   # Single/batch file processing, exports, highlight clips
│   ├── config.rs            # Config struct, presets, merge logic, VideoResolution::parse_name
│   ├── editor.rs            # Video trimming, stabilization, color correction, boxblur
│   ├── exporter.rs          # SRT, FCPXML, EDL, chapter exporters
│   ├── hwaccel.rs           # Hardware acceleration enum (NVENC, VAAPI, etc.)
│   ├── ml.rs                # Auto-reframe, person segmentation (tract-onnx)
│   ├── preset_rules.rs      # Filename-based preset matching
│   ├── preview.rs          # Low-res preview generation
│   ├── progress.rs          # Batch progress tracking (JSON state file)
│   ├── scene_detection.rs   # FFmpeg-based scene detection wrapper
│   ├── stt_analyzer.rs      # Whisper STT transcription (candle)
│   ├── tests_common.rs     # Shared test helpers (create_test_video, etc.)
│   ├── thumbnail.rs         # Best-frame thumbnail extraction
│   ├── utils.rs             # Path helpers, FFmpeg checks, temp RAII (atomic counter)
│   ├── watermark.rs         # Text/image watermark overlay
│   ├── watch.rs            # Watch loop with stop flag for graceful shutdown
│   ├── gui/                 # Egui-based GUI
│   │   ├── mod.rs           # App struct, sidebar navigation, eframe::App impl
│   │   ├── processing.rs    # Watcher loop, folder config builder, queue worker
│   │   ├── theme.rs         # Dark theme constants (sharp corners, red accent)
│   │   └── tabs/            # UI tab rendering (split from monolithic tabs.rs)
│   │       ├── mod.rs       # Header, folders panel
│   │       ├── settings.rs  # Settings panel (processing, audio, video, exports, advanced)
│   │       ├── modals.rs    # Delete confirm, setup wizard, generic modal
│   │       ├── dashboard.rs # Dashboard, toasts, activity log
│   │       └── queue.rs     # Queue panel, batch processing trigger
├── tests/
│   ├── common/mod.rs         # Integration test helpers (delegates to tests_common)
│   ├── pipeline_integration.rs
│   ├── gui_processing_tests.rs
│   └── ml_integration.rs
├── presets/                 # Named preset TOML files
│   ├── youtube.toml
│   ├── shorts.toml
│   ├── podcast.toml
│   └── minimal.toml
├── Cargo.toml
└── ai-vid-editor.example.toml
```

## Build Commands

```bash
# Build CLI + GUI (default)
cargo build --release

# CLI only (smaller binary)
cargo build --release --no-default-features --features cli

# Development build
cargo build

# Run
cargo run -- [args]

# Tests
cargo test --lib              # Unit tests only
cargo test --all-features     # Include integration tests (requires ffmpeg)
cargo clippy --all-features -- -D warnings   # Lint with deny-warnings
cargo fmt --all               # Format

# Check
cargo check --all-features    # Type-check all feature combos
```

## Key Conventions

### Error Handling
- Use `anyhow::Result<T>` for fallible operations; propagate with `?`
- Never use `.unwrap()` or `.expect()` in production code paths — they panic
- Use `let _ =` only for best-effort cleanup where failure is benign
- Add context with `.context()` or `.with_context(|| ...)` at crate boundaries

### FFmpeg Path Handling
- FFmpeg filter strings accept paths with single quotes embedded: `'path/to/file'`
- Escape paths for filter strings using `utils::escape_ffmpeg_filter_path()`:
  - Replaces `\` → `\\`, `'` → `'\''`
  - Example: `subtitles='{escaped}'`, `fontfile='{escaped}'`
- Never interpolate raw paths into filter strings without escaping

### Config Merge Behavior
- `Config::merge(other)` takes fields from `other` only when they differ from defaults
- Scalar fields (threshold_db, padding, etc.): only override if non-default
- Enum fields: always taken from `other` (explicit variants)
- Vec/Option fields: taken from `other` if present/non-empty
- This means: explicit user config is preserved, presets only fill defaults

### Temporary File/Directory Safety
- Use `TempDir` and `TempFile` from `utils.rs` for RAII cleanup
- On drop, `TempDir` removes the directory tree; `TempFile` removes the file
- Use `.keep()` to opt out of cleanup when needed (e.g., preserve intermediate output)
- For model downloads, use temp file + atomic rename to avoid TOCTOU

### Thread Safety in Batch Processing
- `BatchProgress` uses `Arc<Mutex<...>>` — lock with `.lock().unwrap_or_else(|p| p.into_inner())`
- Never assume mutex is not poisoned; handle poison gracefully
- Use `Ordering::SeqCst` for atomic counter operations

### Naming
- `from_str` / `parse_name`: avoid naming methods `from_str` to prevent collision with `std::FromStr` trait. Use `parse_name` instead.
- `HwAccel::parse_name`, `WatermarkPosition::parse_name`, `Preset::parse_name`, `VideoResolution::parse_name`
- Resolution parsing is centralized in `VideoResolution::parse_name()` — do not duplicate match arms in `main.rs` or elsewhere

### Module Re-exports (lib.rs)
- Public re-exports allow binary and tests to access library modules uniformly
- Preview module must be explicitly re-exported: `pub use preview;`
- `tests_common` module is public (not gated by `#[cfg(test)]`) so integration tests can use `ai_vid_editor::tests_common::create_test_video`
- Both `lib.rs` and `main.rs` declare `pub mod tests_common;` so `crate::tests_common` works in both compilation targets

### GUI Theme (theme.rs)
- Corner radius is intentionally `0.0` everywhere — sharp rectangular edges, no rounding
- Red accent (`ACCENT_PRIMARY = rgb(230,57,70)`) on dark background (`PANEL_BG = rgb(14,14,16)`)
- Sidebar active state: red-tinted background + red border stroke (no accent column that shifts content)
- When adding new UI elements, use the theme constants — never hardcode colors or radii

### Graceful Shutdown
- Watch modes (`run_watch_mode`, `run_multi_watch_mode`) register a `ctrlc` handler that sets a shared `AtomicBool` stop flag
- `WatchFolderConfig` carries a `stop: &AtomicBool` — `run_watch_loop` checks it each iteration and breaks with `Ok(())`
- Multi-watch mode joins all watcher threads on shutdown before returning
- GUI watcher uses a bounded `sync_channel(1000)` — blocks sender if GUI thread stalls, preventing unbounded memory growth

## Critical Code Locations

### Auto-reframe
- `ml.rs:analyze_video()` — Extracts frames, runs face detection, builds crop regions
- `ml.rs:generate_crop_filter()` — Generates FFmpeg filter with smoothed linear interpolation
- `ml.rs:CropRegion::from_face()` — Computes 9:16 crop centered on face, guards against zero/inf aspect

### Silence Processing
- `analyzer.rs:detect_silence()` — FFmpeg silence detection with JSON output parsing
- `batch_processor.rs:merge_silences_and_scenes()` — Merges silence + scene boundaries, deduplicates overlapping segments
- `editor.rs:trim_video_with_progress()` — Chunked trimming to avoid FFmpeg arg-length limits

### Clip Extraction
- `batch_processor.rs:extract_highlight_clips()` — Uses `-ss` before `-i` with `-c copy` for fast keyframe-seeking
- Cuts align to nearest keyframe (acceptable for highlight clips); add `-avoid_negative_ts make_zero` to prevent timestamp issues
- For frame-accurate cuts, re-encode instead of stream copy

### Transcription
- `stt_analyzer.rs` — Whisper via candle, downloads model on first use
- `pcm_to_mel()` — FFT-based mel spectrogram (rustfft), handles short audio gracefully
- Audio loaded entirely into memory; no streaming for simplicity

### Config Serialization
- `Config::generate_default_toml()` serializes to TOML, then rounds all floats to 2 decimal places via `round_floats_in_value()` (TOML Value tree walk)
- This avoids f32→f64 serialization artifacts (e.g., `0.10000000149011612`)
- `VideoConfig` has a manual `Default` impl — serde defaults and Rust `Default` must stay in sync

### Config Priority
1. CLI flags (highest)
2. Config file fields (explicit values)
3. Preset values (only if non-default)
4. Default values (lowest)

## Default Behavior (CLI vs GUI)

When running `ai-vid-editor` with no arguments:
- **GUI launches by default** (when compiled with `gui` feature)
- Use `--headless` flag to enter watch/daemon mode instead
- Use `--gui` flag to explicitly request GUI (same as default)

```
cargo run              # Launches GUI (default)
cargo run -- --headless # Launches watch mode (if watch folders configured)
cargo run -- --gui      # Explicit GUI (same as default)
```

The old behavior (TTY detection → fallback to watch mode) has been removed. GUI is now the unconditional default.

## Feature Flags

| Flag | Enables |
|------|---------|
| `cli` | Clap CLI parsing, notify-rust desktop notifications, ctrlc signal handling |
| `gui` | Egui-based GUI, rfd file dialogs, chrono |
| `notify-rust` | Desktop notifications (auto-enabled with cli) |

Default: `cli` + `gui`

## Known Limitations

- Audio for transcription is loaded entirely into RAM (no streaming)
- Auto-reframe uses linear interpolation between smoothed keyframes (not full piecewise)
- Temp directories for ML frame extraction are cleaned on scope exit; may leak on panic
- `blur_background` applies a uniform boxblur to the entire video (no ML person segmentation yet) — see `ml::BackgroundBlurProcessor` for the intended implementation
- Highlight clip extraction uses stream copy (`-c copy`) so cuts align to keyframes, not exact timestamps
- FFmpeg concat demuxer path escaping handles `'` and newlines but not all special characters
