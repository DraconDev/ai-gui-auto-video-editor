# Project State

## Current Focus
Add per-folder processing settings UI with toggles, sliders, and dropdowns for reframing, audio, video, and export options.

## Completed
- [x] Introduce Auto-Reframe (9:16) toggle to crop to vertical for Shorts/Reels/TikTok, persisted per folder.
- [x] Add Blur Background toggle for portrait reframing, persisted per folder.
- [x] Add Scene Detection toggle to refine edit points at scene changes, persisted per folder.
- [x] Add Silence Threshold slider (-60 to -10 dB) to control ambient speech retention, persisted per folder.
- [x] Add Audio section with Enhance Audio toggle, Noise Reduction toggle, and Target Loudness (-24 to -6 LUFS) slider.
- [x] Add Video Output section with GPU Encoding dropdown (None/NVENC/AMF/VAAPI/VideoToolbox) and Target Resolution selector (720p–4K + vertical).
- [x] Add Exports section with toggles for Subtitles, Chapters, Captions, Clips, Preview, and Multi‑format output.
- [x] Wire all controls to folder-level settings with dirty tracking and save propagation.
