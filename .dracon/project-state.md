# Project State

## Current Focus
Replace `remove_silence` setting with `silence_mode` using conditional mapping to `SilenceMode::Cut` or `SilenceMode::Keep`.

## Completed
- [x] Rename `remove_silence` to `silence_mode` in FolderSettings struct
- [x] Add conditional logic mapping to `SilenceMode::Cut` when `setup_remove_silence` is true, otherwise `SilenceMode::Keep`
- [x] Update default construction to include the new `silence_mode` field
