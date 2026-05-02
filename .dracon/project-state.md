# Project State

## Current Focus
Added a helper function to map silence mode variants to human-readable names.

## Context
This change improves code organization by centralizing the mapping of `SilenceMode` variants to their display names, making the code more maintainable and consistent with the existing `yes_no` helper.

## Completed
- [x] Added `silence_mode_name` function to convert `SilenceMode` variants to display strings
- [x] Implemented matching logic for all variants: `Keep`, `Cut`, and `Speedup`

## In Progress
- [ ] None

## Blockers
- None

## Next Steps
1. Verify the new function works correctly with existing UI components
2. Consider adding similar helpers for other enum types if needed
