# Project State

## Current Focus
Remove legacy per‑folder settings UI and associated logic, simplifying the settings panel to static text rendering.

## Completed
- [x] Refactor settings extraction and rendering logic in `src/gui/tabs.rs`, eliminating duplicated toggle and slider code for per‑folder audio, video, and export settings.
- [x] Consolidate state handling into a single section, reducing code duplication and improving maintainability.
