#Project State

## Current Focus
Fix validation warning and prevent saving invalid clip duration settings when min > max

## Completed
- [x] Implemented new variables (`new_clip_min`, `new_clip_max`) to track slider values before validation
- [x] Added logic to revert invalid settings to original values when min duration exceeds max duration
- [x] Updated validation warning message to reflect new variable names
