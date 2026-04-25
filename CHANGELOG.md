# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.76.0] - 2026-04-25

### Added
- **Preview export wiring**: `--preview` CLI flag now generates a 30s/480px low-res preview file alongside the main output via `preview::generate_preview()`
- **Batch progress persistence**: `BatchProgress` tracks completed/failed files in a JSON state file; batch processing skips already-processed files and resumes from where it left off
- **Parallel batch resume support**: `process_batch_dir_parallel()` filters out completed files before spawning workers, with mutex-protected progress saves

### Changed
- **GUI modularization**: Split monolithic `gui.rs` (2149 lines) into `gui.rs` (types + App struct), `gui/processing.rs` (watcher loop + folder config), and `gui/tabs.rs` (all draw methods)

## [0.68.8] - 2026-04-25

### Fixed (Critical)
- **Proc::new typo** in `batch_processor.rs:815` — changed to `Command::new` preventing highlight clip extraction crash
- **Watermark position hardcoded** to `BottomRight` — now correctly parses config/CLI `--watermark-position`
- **Hardcoded font path** `/usr/share/fonts/...` — now tries 10+ system font paths with recursive fallback
- **Unicode truncation** in chapter titles — byte slicing `&title[..50]` → character-safe `chars().take(50)`
- **XML escaping** in FCPXML export — filenames with `&`, `<`, `>`, `"` now produce valid XML
- **`anyhow::Context` import placement** in `analyzer.rs` — moved from mid-file to module top
- **`SystemTime::duration_since` silent failure** — `unwrap_or_default()` → `expect()` with clear message

### Fixed (High)
- **Release build optimization** — `opt-level = "s"` → `opt-level = 2` for better video processing speed
- **Config::merge() missing fields** — added `scene_detect`, `scene_threshold`, `watermark`, `watermark_scale`, `watermark_position`, `thumbnail`, `multi_format`, `extra_resolutions`

### Fixed (Medium)
- **WalkDir depth limit** — added `.max_depth(5)` to prevent filesystem traversal
- **Thumbnail frame sampling** — reduced from 1fps to 0.2fps (1/5th the IO for long videos)
- **WatermarkPosition::from_str()** — added with tests for position parsing

## [0.38.0] - 2026-04-25

### Added
- **Auto-thumbnail generation**: Extracts and scores candidate frames to generate YouTube-ready thumbnails
- **Smart scene-change detection**: Uses ffmpeg scene detection for smarter cuts beyond silence
- **Watermark/logo overlay support**: Image and text watermarks with 5 position options
- **Quick preview generation**: Low-res preview renders for fast review before full export
- **Multi-format output**: Simultaneous export to multiple resolutions (720p, 1080p, 4K, vertical)
- **Social media presets**: TikTok, Instagram Reels, Twitter/X presets with platform-optimized settings
- **Per-file preset selection**: Filename pattern matching for automatic preset assignment
- **Video resolution targeting**: Configurable output resolution per preset
- **Parallel batch processing**: Process multiple videos simultaneously with configurable worker threads
- **Batch job persistence**: Resume interrupted batch jobs from where they left off
- **Config validation**: Warns about incompatible feature combinations
- **58 new tests**: Comprehensive test coverage for all new features

### Changed
- **Major bug fixes**: 32 bugs fixed including loudnorm parsing, ffmpeg arg formatting, race conditions
- **Performance**: Reduced allocations, optimized ffmpeg commands, chunk-based trimming
- **Error handling**: Replaced `unwrap()`/`expect()` with proper error propagation throughout
- **GUI stability**: Fixed atomic ordering, channel disconnect handling, watcher thread races
- **Safety**: Removed `unsafe { libc::isatty }` in favor of `std::io::IsTerminal`

### Fixed
- `parse_loudnorm_stats` pattern matching with spaces around colons
- `concatenate_videos` broken ffmpeg argument formatting
- `generate_duck_filter` hardcoded volume (now accepts parameter)
- `stabilize` shared temp file race condition
- `export_edl` all-zero timestamps
- `export_fcpxml` hardcoded 1-hour duration
- `config::merge()` asymmetric boolean propagation
- `center_crop_9_16` aspect ratio math
- ASS subtitle backslash escaping
- `extract_highlight_clips` duration clamping
- `burn_subtitles_into_video` silent success on missing output
- `format_srt_time` millisecond carry bug
- `parse_ffmpeg_silence` negative-duration validation
- `truncate_path` underflow for small max_len
- `reframe()` silent fallback to 1920x1080

## [0.21.4] - 2025-06-15

### Added
- Initial release with core video editing pipeline
- Silence detection and removal (cut/speedup modes)
- Audio enhancement (two-pass loudnorm + EQ)
- Video stabilization, color correction, auto-reframe
- Batch processing and watch folder mode
- GUI built with egui
- Export formats: SRT, FCPXML, EDL, YouTube chapters
- Preset profiles: YouTube, Shorts, Podcast, Minimal
- Whisper-based speech-to-text and filler word removal

[Unreleased]: https://github.com/DraconDev/ai-vid-editor/compare/v0.68.8...HEAD
[0.68.8]: https://github.com/DraconDev/ai-vid-editor/compare/v0.38.0...v0.68.8
[0.38.0]: https://github.com/DraconDev/ai-vid-editor/compare/v0.21.4...v0.38.0
