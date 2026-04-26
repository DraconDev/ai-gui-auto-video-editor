# Project State

## Current Focus
Added support for starting the GUI minimized via a new constructor parameter

## Completed
- [x] Added `start_minimized` field to `App` struct to control initial window state
- [x] Added `first_frame` flag to handle window minimization on first render
- [x] Implemented logic to hide window if `start_minimized` is true on first frame
- [x] Updated constructor to accept `start_minimized` parameter
