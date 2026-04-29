# Project State

## Current Focus
Add comprehensive per‑folder processing options covering silence handling, clipping, watermarks, intros/outros, and export formats.

## Completed
- [x] Extend `FolderSettings` with silence parameters (`min_duration`, `padding`, `mode`, `speedup_factor`, `min_silence_for_speedup`, `scene_threshold`) and optional clip settings.
- [x] Add clip, watermark, intro/outro, music, duck volume, export flags (`fcpxml`, `edl`, `thumbnail`) to `FolderSettings`.
- [x] Update GUI to expose the new silence mode selector, sliders for padding, min duration, speed‑up factor, and minimum speed‑up duration.
- [x] Implement conditional UI logic to show speed‑up controls only when silence mode is set to `Speedup`.
- [x] Ensure all new settings default to sensible values and are correctly persisted and loaded via serialization.
