# Project State

## Current Focus
Add comprehensive regression tests for utility functions including case‑insensitive video detection, ffmpeg filter path escaping, temporary directory/file lifecycle, and nested directory traversal with depth limits.

## Completed
- [x] Test that `is_video_file` recognizes files with any case extension (e.g., `.MP4`, `.mOv`, `.AVI`)
- [x] Test that `escape_ffmpeg_filter_path` leaves simple paths unchanged
- [x] Test that `escape_ffmpeg_filter_path` correctly escapes single quotes in paths
- [x] Test that `escape_ffmpeg_filter_path` correctly escapes backslashes in Windows‑style paths
- [x] Test that `escape_ffmpeg_filter_path` escapes both single quotes and backslashes when both are present
- [x] Test that `TempDir::new` creates the temporary directory and its path exists
- [x] Test that dropping a `TempDir` without `keep()` removes the directory on drop
- [x] Test that calling `keep()` on a `TempDir` prevents cleanup when `into_path` is used
- [x] Test that `TempFile` removes its file automatically when dropped
- [x] Test that `find_video_files` discovers a video file located deep within nested directories
- [x] Test that `find_video_files` respects the `max_depth` parameter and excludes files beyond depth 10
No other sections or content are required.
