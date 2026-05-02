# Project State

## Current Focus
Refactored `yes_no` helper method to be called via `Self` for consistency in the tabs module.

## Context
The change improves code consistency by ensuring the `yes_no` helper is called consistently through the `App` struct rather than as a standalone function.

## Completed
- [x] Changed `yes_no(folder.settings.stabilize)` to `Self::yes_no(folder.settings.stabilize)`
- [x] Applied same change to `color_correct` and `reframe` settings for uniformity

## In Progress
- [x] Verified no functional changes were introduced

## Blockers
- None identified

## Next Steps
1. Review other similar calls in the module for consistency
2. Ensure no other refactoring opportunities exist in the tabs module
