# Project State

## Current Focus
Fix segment keep positions to ensure they respect `keep_end` and `cut_end` ordering

## Completed
- [x] Replace `current_pos` assignment with `let cut_end = (seg.end - padding).max(0.0);` and compute `current_pos = current_pos.max(keep_end).max(cut_end);`
- [x] Ensure non‑negative segment start by using `max(0.0)` for `cut_end` and proper max ordering
