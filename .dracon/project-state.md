# Project State

## Current Focus
Adjust clip duration sliders to ensure min clips are at least 30 seconds and drop validation‑revert logic

## Completed
- [x] Remove original_clip_min and original_clip_max assignments and the revert logic
- [x] Remove UI warning and revert code when min exceeds max
- [x] Update slider range to clamp min duration to ≥30 seconds using `new_clip_min.max(30.0)..=300.0`
