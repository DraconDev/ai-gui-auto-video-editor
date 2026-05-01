# Project State

## Current Focus
Add optional speech-to-text transcription for filler-word removal planning in batch processing

## Completed
- [x] Add `maybe_transcribe_for_filler_words` function that transcribes input files when `config.filler_words.enabled` is true
- [x] Import `calculate_keep_segments_from_transcript` from the editor module for transcript-based segment calculation
- [x] Implement graceful fallback: log warning and return `None` on transcription failure, allowing silence-based processing to proceed
