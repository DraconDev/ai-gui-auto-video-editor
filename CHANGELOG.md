# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [19.61.0] - 2026-05-23

### Features
- **ML Background Blur (Person Segmentation)**: Integrated `PersonSegmenter` (MODNet ONNX via tract-onnx) into the video pipeline. When `blur_background` is enabled with `ml_blur_strength > 0.0`, a second pass keeps the person sharp while blurring the background. Config: `ml_blur_strength` (sigma, default 15.0) and `ml_inference_scale` (downscale, default 0.5 for 2× speedup). GUI toggle in Video Output settings with strength and scale sliders. Audio preserved.

## [19.60.4] - 2026-05-23

### Bug Fixes
- **Loudnorm JSON parse truncation** (`editor.rs:751`): `str::find('}')` found the `}` inside the string value `"dynamic"` in FFmpeg's output, truncating JSON and losing the `target_offset` field. This caused over-correction and distorted audio. Fixed by switching to `str::rfind('}')` to find the actual closing brace.
- **BatchProgress mtime comparison logic bug** (`progress.rs:44-65`): Stored elapsed seconds from `SystemTime::elapsed()` instead of raw mtime. Age comparison failed if more than 5 seconds passed between `mark_completed()` and `is_completed()`. Fixed by storing raw `SystemTime` and using `duration_since()` with 5s tolerance.
- **Join path filename collision** (`editor.rs:648-656`): Unpadded `{:x}` format on u128 nanos could produce filenames exceeding `NAME_MAX` on some systems. Fixed with `{:016x}` for 16-char zero-padded hex and `u128::MAX` guard.
- **Silent FPS default hides parse failures** (`ml.rs:121-136`): `parse::<f32>().unwrap_or(25.0)` silently returned 25.0 on malformed FPS strings, causing downstream processing failures. Fixed with `.context()` error propagation.

### Code Quality
- **552 unit tests passing**: All tests pass, clippy clean (`-D warnings`), `cargo fmt` clean
- **7 useless comparisons removed**: `assert!(len() >= 0)` in `editor.rs` and `batch_processor.rs` replaced with meaningful `debug_assert!` bounds
- **Unused imports cleaned**: Removed unused `use toml::toml` (config.rs), `FolderSettings` import (processing.rs), `std::process::Command` (watermark.rs tests)
- **`mut` on immutable vectors removed** (`exporter.rs:1015,1089`)
- **`mod tests` structure fixed** (`watermark.rs`): Missing `mod tests {` opening brace caused compilation failure
- **`stt_analyzer.rs` float sort** (`stt_analyzer.rs:600`): `partial_cmp(...).unwrap()` replaced with `sort_by(|a,b| a.start.to_bits().cmp(&b.start.to_bits()))` for consistent NaN handling
- **ANSI color codes conditional** (`batch_processor.rs:700-715`): Colors only enabled when stdout is a TTY (via `std::io::IsTerminal`)

### Features
- **Panic-free production code**: 0 `panic!` in production paths; 2 previous `panic!` calls in `hwaccel.rs` tests replaced with `assert_eq!`

### GUI
- **Sidebar centering**: Navigation and settings buttons now centered with equal spacing (10px each side) in the 180px sidebar

## [19.59.0] - 2026-05-17