# Project State

## Current Focus
Add UI for video joining settings (join mode and after‑count) to allow users to configure how videos are concatenated.

## Completed
- [x] Added UI section "Video Joining" with a dropdown selector for join mode.
- [x] Implemented logic to update folder settings when the selected join mode changes.
- [x] Added conditional UI slider for "Join After Count" that appears only when mode is After Count, updating the count value.
- [x] Introduced extraction of `join_mode` and `join_after_count` from folder settings with appropriate default handling.
- [x] Added explanatory label indicating that "Off = no joining. After Count = join every N videos."
