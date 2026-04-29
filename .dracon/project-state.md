# Project State

## Current Focus
Add settings navigation keyboard shortcuts and adaptive UI repaint logic

## Completed
- [x] Add navigate_settings_category helper method to cycle through settings categories (Processing, Audio, Video, Exports, Advanced) with delta-based index wrapping
- [x] Add cross-platform keyboard shortcuts for Settings/All tabs: Ctrl+S to save config and show success toast, Ctrl+Left/Right arrows to navigate categories, Ctrl+1-5 to jump to specific categories
- [x] Replace fixed 250ms UI repaint with adaptive logic: repaint immediately when processing or watching, else use 250ms interval to reduce idle CPU usage
- [x] Refresh Cargo.lock dependency lockfile
