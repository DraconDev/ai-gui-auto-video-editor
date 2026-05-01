# Project State

## Current Focus
Updated test expectations in `src/editor.rs` to reflect new segment boundaries after refactoring filler‑word aware transcript segmentation.

## Completed
- [x] Adjusted `assert_eq!` statements in `src/editor.rs` to set `processed[0].end` to 2.0, `processed[1].start` to 2.0, and `processed[1].end` to 2.1, matching the revised segmentation logic.
