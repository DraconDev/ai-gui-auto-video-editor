# Project State

## Current Focus
Ready for v19.1.5 release. CHANGELOG updated, needs tag + commit.

## Completed
- [x] fix(filler_words): Wire filler_words pipeline to Whisper transcription via `maybe_transcribe_for_filler_words()`
- [x] fix(editor.rs): Rewrite `calculate_keep_segments_from_transcript` to correctly handle filler segments
- [x] test(pipeline): Add 16 new pipeline integration tests (Tier 1-3)
- [x] test(common): Add `create_speech_video()` and `test_speech_video_path()` helpers
- [x] docs(CHANGELOG): Document v19.1.5 changes
