# AI Video Editor - Agent Reference

## Project Overview

AI-powered video editor for content creators. Processes raw footage through automated pipelines: silence detection/cutting, audio enhancement, scene-change detection, auto-reframe, and more. Supports both CLI and GUI modes from a single binary.

## Repository Structure

```
ai-vid-editor/
├── src/
│   ├── main.rs              # CLI entry point, arg parsing, watch mode
│   ├── lib.rs               # Library exports
│   ├── analyzer.rs          # Silence detection, scene detection
│   ├── batch_processor.rs   # Single/batch file processing, exports
│   ├── config.rs            # Config struct, presets, merge logic
│   ├── editor.rs            # Video trimming, stabilization, color correction
│   ├── exporter.rs          # SRT, FCPXML, EDL, chapter exporters
│   ├── hwaccel.rs           # Hardware acceleration enum (NVENC, VAAPI, etc.)
│   ├── ml.rs                # Auto-reframe, person segmentation (tract-onnx)
│   ├── preset_rules.rs      # Filename-based preset matching
│   ├── preview.rs          # Low-res preview generation
│   ├── progress.rs          # Batch progress tracking (JSON state file)
│   ├── scene_detection.rs   # FFmpeg-based scene detection wrapper
│   ├── stt_analyzer.rs      # Whisper STT transcription (candle)
│   ├── thumbnail.rs         # Best-frame thumbnail extraction
│   ├── utils.rs             # Path helpers, FFmpeg checks, temp RAII
│   ├── watermark.rs         # Text/image watermark overlay
│   ├── gui/                 # Egui-based GUI
│   │   ├── mod.rs
│   │   ├── processing.rs    # Watcher loop, folder config builder
│   │   └── tabs.rs          # UI tab rendering
├── tests/
│   ├── common/mod.rs         # Shared test helpers (ffmpeg presence, test videos)
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
- `HwAccel::parse_name`, `WatermarkPosition::parse_name`, `Preset::parse_name`

### Module Re-exports (lib.rs)
- Public re-exports allow binary and tests to access library modules uniformly
- Preview module must be explicitly re-exported: `pub use preview;`

## Critical Code Locations

### Auto-reframe
- `ml.rs:analyze_video()` — Extracts frames, runs face detection, builds crop regions
- `ml.rs:generate_crop_filter()` — Generates FFmpeg filter with smoothed linear interpolation
- `ml.rs:CropRegion::from_face()` — Computes 9:16 crop centered on face, guards against zero/inf aspect

### Silence Processing
- `analyzer.rs:detect_silence()` — FFmpeg silence detection with JSON output parsing
- `batch_processor.rs:merge_silences_and_scenes()` — Merges silence + scene boundaries, deduplicates overlapping segments
- `editor.rs:trim_video_with_progress()` — Chunked trimming to avoid FFmpeg arg-length limits

### Transcription
- `stt_analyzer.rs` — Whisper via candle, downloads model on first use
- `pcm_to_mel()` — FFT-based mel spectrogram (rustfft), handles short audio gracefully
- Audio loaded entirely into memory; no streaming for simplicity

### Config Priority
1. CLI flags (highest)
2. Config file fields (explicit values)
3. Preset values (only if non-default)
4. Default values (lowest)

## Feature Flags

| Flag | Enables |
|------|---------|
| `cli` | Clap CLI parsing, notify-rust desktop notifications |
| `gui` | Egui-based GUI, rfd file dialogs, chrono |
| `notify-rust` | Desktop notifications (auto-enabled with cli) |

Default: `cli` + `gui`

## Known Limitations

- Audio for transcription is loaded entirely into RAM (no streaming)
- Auto-reframe uses linear interpolation between smoothed keyframes (not full piecewise)
- Temp directories for ML frame extraction are cleaned on scope exit; may leak on panic
- FFmpeg concat demuxer path escaping handles `'` and newlines but not all special characters
