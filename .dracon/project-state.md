# Project State

## Current Focus
Refactor folder‑selection handling to switch from `preset` string to `settings` of type `FolderSettings`

## Completed
- [x] Replaced `preset` extraction with `settings` extraction from the selected folder
- [x] Updated `QueuedFile` construction to store `settings` instead of `preset`
- [x] Removed reference to `preset` in the batch queue push
- [x] Adjusted output directory fallback logic to use the folder’s `output` field via `folder.map(|f| f.output.clone())`
