# Project State

## Current Focus
Add comprehensive settings sidebar panels for audio, video, and export categories, providing interactive controls (toggles, sliders, dropdowns) that update per‑folder settings and mark the project as needing save.

## Completed
- [x] Implemented `draw_settings_audio` function that renders audio settings (enhance audio toggle, noise reduction toggle, target loudness slider) and persists changes to folder settings.
- [x] Implemented `draw_settings_video` function that renders video output settings (hardware acceleration dropdown, target resolution dropdown) and persists changes.
- [x] Implemented `draw_settings_exports` function that renders export options (subtitles, chapters, captions, clips toggles) and persists changes.
- [x] Added UI sections with appropriate labels and spacing to organize settings within the sidebar.
- [x] Integrated per‑folder indexing to ensure settings modifications apply to the correct folder.
