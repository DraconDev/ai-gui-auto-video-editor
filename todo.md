# Audio/Video Options Audit

## Status: ✅ Research Complete, Changes Applied

---

## ✅ Completed

### `enhance_audio` (loudnorm)
- **Fixed**: JSON parse truncation bug (`rfind('}')`)
- **Safe defaults**: highpass=60, TP=-2.0, LRA=7, no EQ, -14 LUFS

### Silence Speedup
- **Off by default** across all presets
- `Speedup` mode still available in GUI
- Default `speedup_factor=2.0x`

### `noise_reduction` (afftdn)
- **Changed**: `nr=15` → `nr=12` (FFmpeg default, preserves voice harmonics)
- **File**: `src/editor.rs:393`

### `duck_volume`
- **Changed**: `0.2` → `0.15` (-16dBFS during speech, research-backed)
- **File**: `src/config.rs:283`

### `threshold_db`
- **Kept**: `-30dB` (standard for voice, FFmpeg default -60dB too sensitive)

### Loudnorm TP
- **Kept**: `-2.0` (more conservative than YouTube's -1.5, safer for unknown source quality)

---

## 🔜 Future Audit Items (lower priority)

- [ ] Auto-reframe confidence threshold (`src/ml.rs`)
- [ ] Stabilization settings (`src/editor.rs:429`)
- [ ] Scene detection threshold (`src/config.rs`)
- [ ] Video quality CRF values (`src/editor.rs`)
- [ ] Color correction (currently pass-through)

---

## References

- FFmpeg afftdn docs: ayosec.github.io/ffmpeg-filters-docs/8.1/Filters/Audio/afftdn.html
- YouTube LUFS target: -14 LUFS, -1.5dBTP (2025)
- Apple Podcasts: -16 LUFS, -1.0dBTP
- Music ducking: -18 to -25dBFS during speech
- Speedup: 1.5x sweet spot, 2x acceptable for deliberate speakers