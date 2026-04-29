# Project State

## Current Focus
Adjust CLI behavior to always launch the GUI by default when compiled with GUI support, and provide clear help instructions when GUI is unavailable. Introduce a dedicated headless mode that bypasses terminal detection.

## Completed
- [x] Removed terminal detection (`IsTerminal`) and simplified condition to always launch GUI when compiled with GUI support.
- [x] Updated headless mode logic to directly run watch mode when enabled, eliminating unnecessary TTY checks.
- [x] Simplified help output for non‑GUI builds, guiding users to use `--headless` or provide input arguments.
- [x] Added user‑friendly message when GUI is launched, advising use of `--headless` for daemon mode.
- [x] Cleaned up and commented code related to GUI launch and headless behavior for better readability.
