# Project State

## Current Focus
Improved drag-and-drop handling for video files in the Queue tab

## Context
The previous implementation of drag-and-drop file handling was too generic. This change makes it more specific to the Queue tab and simplifies the default values used when processing dropped files.

## Completed
- [x] Restricted drag-and-drop file handling to only work on the Queue tab
- [x] Simplified default values for output directory, preset, and settings when processing dropped files
- [x] Updated comment to reflect the more specific functionality

## In Progress
- [ ] No active work in progress

## Blockers
- None identified

## Next Steps
1. Verify the new behavior works as expected with various file types
2. Consider adding visual feedback when files are successfully dropped onto the Queue tab
