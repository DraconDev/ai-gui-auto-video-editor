# Project State

## Current Focus
feat(silence): enable Keep mode to preserve all audio without cutting silences and adjust config handling accordingly

## Completed
- [x] add Keep variant to SilenceMode and handle it in editor processing
- [x] modify calculate_keep_segments to return full‑duration segment when mode is Keep
- [x] update GUI config builder to use folder.settings.silence_mode instead of remove_silence flag
