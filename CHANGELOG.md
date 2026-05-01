# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [19.19.0] - 2026-05-01

### UX Improvements
- **Memory leak fixes**: Capped `activity_log` to 500 entries (FIFO eviction), replaced unbounded `HashSet` in watcher with bounded `VecDeque` (max 10,000), auto-clear completed batch queue items after 60s (max 100 items)
- **Keyboard shortcuts**: Added global Ctrl+1-5 for tab navigation, Ctrl+S for save, Ctrl+ArrowLeft/Right for settings category navigation, Ctrl+Shift+1-5 for direct category access
- **Retry button**: Failed batch queue items now show a "Retry" button to re-queue them
- **Re-run Setup Wizard**: Added button in Settings sidebar to re-launch the first-run wizard
- **Drag-and-drop**: Drop video files onto the Queue tab to add them to the batch
- **Recent outputs**: Folders panel shows last 5 processed output files with hover tooltips

### Refactor
- Moved `use clap::Parser` under `#[cfg(feature = "cli")]` guard in main.rs
- Removed duplicate `VIDEO_EXTENSIONS` constant from `analyzer.rs`

## [19.14.2] - 2026-05-01

### UX Improvements
- **Memory leak fixes**: Capped `activity_log` to 500 entries (FIFO eviction), replaced unbounded `HashSet` in watcher with bounded `VecDeque` (max 10,000), auto-clear completed batch queue items after 60s (max 100 items)
- **Keyboard shortcuts**: Added global Ctrl+1-5 for tab navigation, Ctrl+S for save, Ctrl+ArrowLeft/Right for settings category navigation, Ctrl+Shift+1-5 for direct category access
- **Retry button**: Failed batch queue items now show a "Retry" button to re-queue them
- **Re-run Setup Wizard**: Added button in Settings sidebar to re-launch the first-run wizard

### Refactor
- Moved `use clap::Parser` under `#[cfg(feature = "cli")]` guard in main.rs
- Removed duplicate `VIDEO_EXTENSIONS` constant from `analyzer.rs`

## [19.3.10] - 2026-05-01

### Refactor
- Moved `use clap::Parser` under `#[cfg(feature = "cli")]` guard in main.rs so `--no-default-features --features gui` compiles correctly
- Removed duplicate `VIDEO_EXTENSIONS` constant from `analyzer.rs` (already defined in `utils.rs`)

## [19.2.2] - 2026-05-01

### Maintenance
- Internal Cargo.lock updates (no functional changes since v19.1.5)
- Fixed release workflow Windows artifact upload path (.tar.gz → also .zip)
- Enabled integration tests in CI (`cargo test --all-features`)
- Added CHANGELOG comparison links for v13.2.0, v14.4.1, v19.1.5, v19.1.8

## [19.1.5] - 2026-05-01

### Fixed (Pipeline)
- **Dead `filler_words` code now functional**: `calculate_keep_segments_from_transcript` in `editor.rs` was rewritten to correctly handle filler word segments with padding. Previously the function was never wired into the pipeline — now `maybe_transcribe_for_filler_words()` in `batch_processor.rs` runs Whisper transcription when `config.filler_words.enabled` and passes transcript to the segment calculator.

### Added (Tests)
- **16 new pipeline integration tests** (tests/pipeline_integration.rs):
  - Tier 1 (8): `test_speedup_in_pipeline`, `test_keep_mode_in_pipeline`, `test_scaling_in_pipeline`, `test_intro_outro_in_pipeline`, `test_multi_resolution_output`, `test_thumbnail_dimensions_in_pipeline`, `test_watermark_in_pipeline_image`, `test_preview_duration_in_pipeline`
  - Tier 2 (6): `test_captions_in_pipeline_with_speech`, `test_subtitles_export_with_speech`, `test_chapters_with_speech`, `test_clips_extraction_with_speech`, `test_audio_ducking_with_speech`, `test_filler_word_removal_pipeline`
  - Tier 3 (2): `test_batch_processing_multiple_files`, `test_batch_progress_persistence`
- **Speech test video helper** (tests/common/mod.rs): `create_speech_video()` and `test_speech_video_path()` using espeak + ffmpeg to generate real speech content for speech-driven pipeline tests

## [14.4.1] - 2026-04-30

### Fixed (GUI)
- **Emoji variation selector causing box glyphs**: Characters with U+FE0F variation selector (⚙️🎙️✂️) were rendering as box+base pairs because the embedded emoji font only has base characters. Stripped variation selectors from all emoji literals (gui.rs, tabs.rs: 4 locations).

### Added (Tests)
- **8 new end-to-end pipeline integration tests** (tests/pipeline_integration.rs):
  - `test_watermark_in_pipeline`: Watermark config through full pipeline
  - `test_background_music_in_pipeline`: Background music + ducking through full pipeline
  - `test_scene_detection_in_pipeline`: Scene detection + silence merge through pipeline
  - `test_full_pipeline_all_features`: All pipeline steps enabled simultaneously
  - `test_exports_through_pipeline`: SRT/chapters/FCPXML/EDL/thumbnail via process_single_file
  - `test_multi_format_export`: Multi-resolution output (720p alternate)
  - `test_clip_extraction`: Highlight clip extraction (requires transcription)
  - `test_config_precedence_with_preset`: Config file + Shorts preset merge, verifies 1080x1920 vertical output
- **2 test helpers** (tests/common/mod.rs): `create_test_audio_file()`, `create_test_watermark_png()`

## [13.2.0] - 2026-04-30

### Fixed (GUI)
- **Emoji icons rendering as empty boxes**: GUI icons (dropdown chevrons ▲/▼, preset icons 🎬, folder icons 📁, checkmarks ✓, remove buttons ✕) were rendering as empty boxes because egui's default font has no emoji support. Added `configure_emoji_fonts()` that loads Noto Color Emoji (Linux), Apple Color Emoji (macOS), or Segoe UI Emoji (Windows) as a fallback font.

### Added (Tests)
- **Comprehensive test expansion**: Added 53 new tests across previously under-tested modules:
  - `analyzer.rs` (+8): Silence parser edge cases (malformed lines, empty output, large floats, unmatched starts/ends, integer timestamps, whitespace)
  - `batch_processor.rs` (+6): `merge_silences_and_scenes` behavior (empty inputs, overlapping, boundary extension, no overlap)
  - `thumbnail.rs` (+4): `parse_entropy` edge cases (negative values, multiple colons, zero), frame extraction at time 0
  - `gui/processing.rs` (+10): `build_folder_config` field mapping, legacy `remove_silence` migration, preset merge, watermark/music path resolution
  - `hwaccel.rs` (+6): Roundtrip parsing, display names, input args, `needs_hwaccel_input`, empty/whitespace edge cases
  - `ml.rs` (+6): `CropRegion::from_face` guard tests (zero/negative/infinite aspect), `center_crop_9_16` wide/narrow/zero video
  - `exporter.rs` (+10): FCPXML escaping, SRT timestamps, EDL structure, YouTube chapters
  - `watermark.rs` (+6): Filter string generation, position math, scaling, font path escaping
  - `editor.rs` (+7): Padding overlap regression tests
  - `utils.rs` (+10): Path utilities, escaping, temp RAII cleanup
  - `scene_detection.rs` (+5): Parsing and threshold sensitivity
  - `preset_rules.rs` (+8): Filename matching with default rules
  - `preview.rs` (+5): Resolution and extension handling
  - `stt_analyzer.rs` (+5): Mel filterbank dimensions and segment handling
- **Integration tests**: Migration logic tests for legacy `remove_silence` → `silence_mode` transition

### Security (Critical)
- **Command injection in FFmpeg filters**: Paths in `subtitles=`, `vidstab=`, and `drawtext=` filter strings were interpolated directly without escaping single quotes. A malicious filename containing `'` could inject arbitrary FFmpeg filter commands. Fixed by adding `escape_ffmpeg_filter_path()` in `utils.rs` that escapes `\`, `'`.

### Fixed (Critical)
- **Caption burn data loss**: `burn_subtitles_into_video()` was renaming the captioned output over the input video, but the original rename logic was broken and could silently destroy the original. Fixed with atomic rename via `atomic_replace()` helper that handles Windows safely.
- **Config merge destroyed base values**: `Config::merge()` was unconditionally overwriting all scalar fields, so merging a default config would destroy user values. Fixed to only take non-default values from `other` as documented.
- **STT panic on short audio**: `pcm_to_mel()` would underflow when computing `n_frames = (pcm.len() - n_fft) / hop_length + 1` if audio was shorter than 25ms (400 samples), causing a panic. Fixed with a guard that returns an empty tensor for short audio.
- **Division by zero in auto-reframe**: `CropRegion::from_face()` computed `crop_width = target_aspect / video_aspect` without checking for zero height, producing `inf`. Fixed with `is_finite()` and `video_height > 0` checks.
- **Auto-reframe ignored face movement**: `generate_crop_filter()` only used the first detected crop region for the entire video, ignoring all intermediate face detections. Fixed with temporal smoothing (moving average) across 5-frame windows and linear interpolation between first/last positions.

### Fixed (High)
- **Silent overlapping segments**: `merge_silences_and_scenes()` could create overlapping silence segments after extending to scene boundaries, causing invalid trim data. Fixed by adding sort + deduplication pass.
- **Missing ffprobe check**: Only `ffmpeg` was checked at startup, but many code paths depend on `ffprobe`. Added `check_ffprobe()` alongside `check_ffmpeg()` at startup.

### Fixed (Medium)
- **ProgressStyle template panic**: `ProgressStyle::template().unwrap()` would panic if the template string was invalid. Fixed with `unwrap_or_else` fallback.
- **Mutex poison panic**: `progress.lock().unwrap()` in parallel batch workers would panic on thread panic (poisoned mutex). Fixed with `lock().unwrap_or_else(|p| p.into_inner())`.
- **Lossy path construction**: 6 instances of `format!("{}.ext", path.display())` could mangle non-UTF-8 filenames. Fixed with `PathBuf::with_extension()` and `OsString::push()`.
- **RAII temp cleanup**: Temp directories in ML frame extraction, thumbnail generation, and vidstab processing were manually cleaned, leaking on early return or panic. Fixed with `TempDir`/`TempFile` RAII wrappers in `utils.rs`.
- **TOCTOU on model download**: Model files could be partially downloaded before another process saw them. Fixed by downloading to `.tmp` then atomically renaming to final path.
- **Case-sensitive font extension**: `find_first_ttf()` only matched lowercase `ttf`, missing `.TTF` on case-sensitive filesystems. Fixed with `to_ascii_lowercase()`.
- **Incomplete concat escaping**: FFmpeg concat demuxer path escaping only handled `'` but not `\n` or `\r`, which could break on filenames with newlines. Fixed by also escaping newlines and carriage returns.

### Added
- **RAII utilities in utils**: `TempDir` (auto-cleanup on drop) and `TempFile` (auto-delete on drop) helpers for safe temp file/directory management.
- **`escape_ffmpeg_filter_path()`**: Sanitizes paths for safe insertion into FFmpeg filter strings by escaping `\`, `'`.

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

[Unreleased]: https://github.com/DraconDev/ai-vid-editor/compare/v19.19.0...HEAD
[19.19.0]: https://github.com/DraconDev/ai-vid-editor/compare/v19.14.2...v19.19.0
[19.14.2]: https://github.com/DraconDev/ai-vid-editor/compare/v19.3.10...v19.14.2
[19.3.10]: https://github.com/DraconDev/ai-vid-editor/compare/v19.2.2...v19.3.10
[19.2.2]: https://github.com/DraconDev/ai-vid-editor/compare/v19.1.8...v19.2.2
[19.1.8]: https://github.com/DraconDev/ai-vid-editor/compare/v19.1.5...v19.1.8
[19.1.5]: https://github.com/DraconDev/ai-vid-editor/compare/v14.4.1...v19.1.5
[14.4.1]: https://github.com/DraconDev/ai-vid-editor/compare/v13.2.0...v14.4.1
[13.2.0]: https://github.com/DraconDev/ai-vid-editor/compare/v0.76.0...v13.2.0
[0.76.0]: https://github.com/DraconDev/ai-vid-editor/compare/v0.68.8...v0.76.0
[0.68.8]: https://github.com/DraconDev/ai-vid-editor/compare/v0.38.0...v0.68.8
[0.38.0]: https://github.com/DraconDev/ai-vid-editor/compare/v0.21.4...v0.38.0
