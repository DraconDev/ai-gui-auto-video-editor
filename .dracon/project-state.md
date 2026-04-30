# Project State## Current Focus
Add end‑to‑end integration tests for pipeline features (watermark, background music, scene detection)

## Completed
- [x] Added `create_test_audio_file` and `create_test_watermark_png` helper functions in `tests/common/mod.rs`
- [x] Added `check_ffmpeg_or_return` and ffprobe utility helpers in `tests/pipeline_integration.rs`
- [x] Added integration test `test_watermark_in_pipeline`
- [x] Added integration test `test_background_music_in_pipeline`
- [x] Added integration test `test_scene_detection_in_pipeline` (and supporting utilities)
