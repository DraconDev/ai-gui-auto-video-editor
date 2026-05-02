# Project State

## Current Focus
Refactored resolution display in folder settings to improve consistency with other settings.

## Context
The change aligns the resolution display format with other settings in the folder summary panel, ensuring uniform handling of optional values.

## Completed
- [x] Updated resolution display to use `unwrap_or_default()` consistently with other settings
- [x] Maintained the same fallback behavior ("—") for missing values

## In Progress
- [x] No active work in progress beyond this change

## Blockers
- None identified

## Next Steps
1. Verify the change doesn't affect any existing resolution display logic
2. Check for any related settings that might need similar formatting updates
