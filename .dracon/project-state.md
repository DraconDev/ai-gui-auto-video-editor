# Project State

## Current Focus
Handle edge cases when removing a watch folder to maintain correct selection and logging

## Completed
- [x] Reset `selected_folder_idx` to 0 or the last valid index when folder removal leaves the list empty or the index out of range
- [x] Log the removal of a watch folder in the activity log
- [x] Auto‑save the configuration after folder removal
