# Project State

## Current Focus
Added input validation to prevent negative time values in timecode formatting functions

## Completed
- [x] Added `seconds.max(0.0)` check in `generate_styled_captions` to prevent negative time values
- [x] Added `seconds.max(0.0)` check in `seconds_to_timecode` to prevent negative time values
