# Project State

## Current Focus
Add `extra_resolutions` configuration support and update generation logic to consider it

## Completed
- [x] Added `extra_resolutions: Option<Vec<VideoResolution>>` field to `FolderSettings` with serialization guard
- [x] Modified `FolderSettings::maybe_generate_local_files` to include `self.extra_resolutions.is_none()` condition
