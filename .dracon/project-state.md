# Project State

## Current Focus
Migrate legacy `remove_silence` boolean to explicit `SilenceMode` with Cut/Keep semantics.

## Completed
- [x] Added migration logic to map `remove_silence` to `SilenceMode::Cut` or `SilenceMode::Keep` in `build_folder_config`
