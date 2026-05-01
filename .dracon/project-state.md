# ProjectState

## Current Focus
Adjust transcript segmentation to apply padding when merging segments near filler words.

## Completed
- [x] Modified `calculate_keep_segments_from_transcript` to set `prev.end = seg.start - padding` and `current_pos = seg.end - padding` for gap handling
- [x] Updated test assertion expecting `processed[1].start` to be `3.0` instead of `2.9` to match new padding logic
