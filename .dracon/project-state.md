# Project State

## Current Focus
Add UI to allow users to select and manage extra video resolutions for folder export settings, updating the folder's configuration when selections change.

## Completed
- [x] Added UI spacing and label "Extra Resolutions" after the multi‑format switch
- [x] Created a horizontal_wrapped list of pill buttons representing the four available resolutions (720p, 1440p, 4K, Vertical 720p)
- [x] Buttons toggle selection, modifying `current_resolutions` and writing the updated list to `folder.settings.extra_resolutions`
- [x] Set `needs_save = true` when the extra‑resolution configuration is changed
---
