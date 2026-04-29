# Project State

## Current Focus
Add step‑based precision and formatting to slider inputs used in the settings UI.

## Completed
- [x] Added `step` parameter to `slider_glow` in `src/gui/theme.rs` and implemented stepping logic and dynamic value formatting.
- [x] Updated `slider_glow` to round values to the nearest step and adjust displayed text based on the step size.
- [x] Modified four calls to `slider_glow` in `src/gui/tabs.rs` (silence threshold, silence padding, silence min duration, and silence speedup factor) to pass the new `step` argument.
