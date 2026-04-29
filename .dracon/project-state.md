# Project State

## Current Focus
Document CLI/GUI default behavior: GUI is now unconditional default; headless/watch mode is opt-in via `--headless`.

## Completed
- [x] Clarify launch defaults in AGENTS.md: `ai-vid-editor` without arguments launches GUI when `gui` feature is enabled; `--headless` enters watch/daemon mode; `--gui` explicitly requests GUI.
- [x] Remove legacy TTY-detection fallback and old watch-mode default, making GUI the unconditional default regardless of terminal state.
