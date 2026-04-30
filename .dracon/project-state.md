# Project State

## CurrentFocus
Dynamic emoji font detection with cross‑platform fallback paths and warning logging on Linux/macOS/Windows

## Completed
- [x] Added `fc-list`‑based emoji font search on Linux and fallback paths for macOS/Windows
- [x] Added `tracing::warn!` when no emoji font is found to aid debugging
- [x] Updated CHANGELOG.md to reflect the fix under "Fixed (GUI)"
- [x] Updated Cargo.lock (binary change) reflecting dependency rebuild
