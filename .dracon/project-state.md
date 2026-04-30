# Project State

## Current Focus
Update 14.4.1 changelog, add 8 new pipeline integration tests with 2 helpers, and fix test ffmpeg availability handling

## Completed
- [x] docs(changelog): Add 14.4.1 release entry with corrected GUI emoji rendering fix noting 4 locations where variation selectors were stripped
- [x] docs(changelog): Document 8 new end-to-end pipeline integration tests and 2 test helpers (create_test_audio_file, create_test_watermark_png) for pipeline testing
- [x] Fix pipeline integration tests failing when ffmpeg/ffprobe are missing by replacing check_ffmpeg() with check_ffmpeg_or_return() to skip tests properly
- [x] Add existence assertions for FCPXML, EDL, and thumbnail exports in test_exports_through_pipeline
- [x] Update test_exports_through_pipeline comments to clarify SRT/chapter exports require transcription for test video content
