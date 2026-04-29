# Project State

## Current Focus
refactor(gui): Remove advanced settings UI panels for audio, video, and export configurations

## Completed
- [x] Remove Auto-Reframe (9:16), Blur Background, Scene Detection, and Silence Threshold setting controls from `draw_settings_advanced`
- [x] Remove `draw_settings_audio` function containing Enhance Audio, Noise Reduction, and Target Loudness controls
- [x] Remove `draw_settings_video` function containing GPU Encoding and Target Resolution dropdown controls
- [x] Remove `draw_settings_exports` function for export format and quality settings
- [x] Simplify `draw_settings_advanced` to return `needs_save` directly after removing 271 lines of UI code
- [x] Update Cargo.lock dependencies
