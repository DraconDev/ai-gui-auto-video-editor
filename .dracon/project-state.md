# Project State

## Current Focus
Refactored silence mode display in folder settings to use a dedicated helper method.

## Context
The change improves consistency by using `Self::silence_mode_name()` instead of directly calling `display_name()` on the silence mode variant.

## Completed
- [x] Replaced direct `display_name()` call with `Self::silence_mode_name()` for silence mode display
- [x] Updated fallback value from `unwrap_or_default()` to `unwrap_or("—")` for clearer empty state representation

## In Progress
- [x] No active work in progress beyond this change

## Blockers
- None identified

## Next Steps
1. Verify the new silence mode display matches expected behavior in UI
2. Consider if similar refactoring should be applied to other settings displays
