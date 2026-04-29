# Project State

## Current Focus
Refactor `queue_worker_loop` to use per‑file folder configuration and pass it to processing functions

## Completed
- [x] Removed legacy analyzer, editor, and duration_getter initializations
- [x] Introduced `FolderState` to capture input path, output directory, preset, enabled flag, and settings
- [x] Built `folder_config` from the new `FolderState` and used it instead of the global `config`
- [x] Replaced global `config.paths.intro`/`config.paths.outro` with `file_config.paths.intro`/`file_config.paths.outro`
- [x] Updated `process_single_file_with_intro_outro_progress` call to receive `&file_config` and the derived paths
- [x] Adjusted variable references to reflect the new configuration flow
