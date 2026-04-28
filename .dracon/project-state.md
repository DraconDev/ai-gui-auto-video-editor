# Project State

## Current Focus
Added dynamic FPS detection for EDL export based on video properties

## Completed
- [x] Added `get_video_fps` method to extract FPS from video using ffprobe
- [x] Updated EDL export to use detected FPS instead of hardcoded 25.0 value
- [x] Implemented fallback to 25.0 FPS when detection fails or path is invalid
