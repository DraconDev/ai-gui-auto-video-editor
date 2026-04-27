# Project State

## Current Focus
Refactored theme glow color to use premultiplied alpha for proper blending

## Completed
- [x] Changed `ACCENT_GLOW` from `from_rgba_unmultiplied` to `from_rgba_premultiplied` for correct alpha blending in the GUI theme
