# Project State

## Current Focus
Introduce UI feedback for no configured folders and lock settings UI while processing is active.

## Completed
- [x] Show a centered placeholder message when the folder list is empty, prompting the user to add a folder.
- [x] Detect when processing is ongoing (queue or active) and replace the settings panels with a locked‑settings notice.
- [x] Preserve existing settings panel rendering for when processing is not active by nesting the original match logic inside a conditional.
