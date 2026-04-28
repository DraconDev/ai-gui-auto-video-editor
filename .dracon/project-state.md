# Project State

## Current Focus
Improved FFmpeg subtitle path handling to prevent command injection vulnerabilities

## Completed
- [x] Added path escaping utility function to handle special characters in FFmpeg filter paths
- [x] Refactored subtitle burning to use escaped paths instead of raw paths in FFmpeg commands
