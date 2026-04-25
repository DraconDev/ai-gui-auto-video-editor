# Project State

## Current Focus
Added preview generation and batch processing improvements to the video processing pipeline

## Completed
- [x] Added preview export capability with `--preview` flag, generating 30s/480px low-res previews
- [x] Implemented batch progress persistence with JSON state tracking for skipped/failed files
- [x] Refactored GUI module into separate files for better organization (types, processing, tabs)
- [x] Added parallel batch resume support with mutex-protected progress saving
