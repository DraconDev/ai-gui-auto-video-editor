# Project State

## Current Focus
Improved drag-and-drop file handling for video files in the Queue tab

## Context
This change enhances the file handling system by properly accessing dropped files through the raw input interface, which provides more reliable access to file paths during drag-and-drop operations.

## Completed
- [x] Updated drag-and-drop file handling to use `i.raw.dropped_files` instead of direct `i.dropped_files` access
- [x] Maintained video file filtering for Queue tab operations

## In Progress
- [x] Implementation of improved drag-and-drop file processing

## Blockers
- No blockers identified for this specific change

## Next Steps
1. Verify the new implementation works consistently across different platforms
2. Test edge cases like multiple simultaneous file drops
