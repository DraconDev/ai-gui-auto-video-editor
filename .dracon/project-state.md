# Project State

## Current Focus
Added configurable video resolution options for output processing

## Completed
- [x] Added `VideoResolution` enum with standard and vertical resolutions (720p, 1080p, 1440p, 4K, vertical variants)
- [x] Implemented resolution dimension lookup via `dimensions()` method
- [x] Added FFmpeg-compatible scale string generation via `to_ffmpeg_scale()`
- [x] Integrated resolution setting into `VideoConfig` with default to 1080p FHD
- [x] Enabled platform-specific resolution presets for social media formats
