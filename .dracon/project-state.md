# ProjectState

## Current Focus
Add unit tests for `find_video_files` to verify nested directory detection and case‑insensitive file extension handling.

## Completed
- [x] Add `test_find_video_files_nested_dirs` that creates video files in a directory and its subdirectory and asserts that two files are found.
- [x] Add `test_find_video_files_case_insensitive` that creates video files with mixed‑case extensions and asserts that three files are found.
