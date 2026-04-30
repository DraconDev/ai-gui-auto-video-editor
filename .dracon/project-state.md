# Project State

## Current Focus
Simplify boolean assertions by removing redundant equality checks.

## Completed
- [x] Replace `assert_eq!(result.export.preview, true);` with `assert!(result.export.preview);` in `test_build_folder_config_preview_override`
- [x] Replace `assert_eq!(result.silence.scene_detect, true);` with `assert!(result.silence.scene_detect);` in `test_build_folder_config_scene_detect_override`
- [x] Replace `assert_eq!(result.export.multi_format, true);` with `assert!(result.export.multi_format);` in `test_build_folder_config_multi_format_override`
- [x] Replace `assert_eq!(result.export.subtitles, true);` with `assert!(result.export.subtitles);` in `test_build_folder_config_subtitles_override`
- [x] Replace the four equality assertions with truthiness checks in `test_build_folder_config_all_settings_at_once`
