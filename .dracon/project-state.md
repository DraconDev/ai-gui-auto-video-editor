# Project State

## Current Focus
Implementing a fixed‑width settings sidebar in the topics tab using `allocate_ui`.

## Completed
- [x] Replaced the old dynamic sidebar drawing with a fixed‑width allocation using `ui.allocate_ui(egui::vec2(sidebar_width, ui.available_height()), …)`.
- [x] Introduced `sidebar_width` and `spacing` constants and calculated `available` and `content_width` for precise layout.
- [x] Added a loop over `SettingsCategory` to generate buttons for Processing, Audio, Video, Exports, and Advanced with active‑state styling.
- [x] Removed the previous vertical layout and manual `ui.add_space(16.0)` handling, consolidating spacing logic.
- [x] Allocated a separate area for the content region using `content_width` to fill the remaining width after the sidebar.
- [x] Updated styling (background, border, corner radius) to reflect active/inactive categories.
