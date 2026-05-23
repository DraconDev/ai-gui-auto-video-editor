# Audio/Video Options Audit

## Status: Research Complete — Pending Implementation

---

## ✅ Completed

### `enhance_audio` (loudnorm)
- **Fixed**: JSON parse truncation bug (`rfind('}')`)
- **Safe defaults**: highpass=60, TP=-2.0, LRA=7, no EQ, -14 LUFS

### Silence Speedup
- **Off by default** across all presets (Shorts, TikTok, Reels → Cut)
- `Speedup` mode still available in GUI
- Default `speedup_factor=2.0x`

---

## 🔍 Research Findings

### `afftdn` noise reduction
- FFmpeg default: `nr=12`
- Current: `nr=15` (too aggressive — can muffle voice)
- **Source**: User reports indicate nr=15 removes harmonics, making voice sound unnatural
- **Recommendation**: Lower to `nr=12`

### `duck_volume`
- Current: `0.2` (-14dBFS)
- Research: Background music should be -18 to -25dBFS during speech
- **Recommendation**: Lower to `0.15` (-16dBFS)

### `threshold_db` (silence detection)
- Current: `-30dB`
- Research: Standard for voice recordings, FFmpeg default (-60dB) is too sensitive
- **Recommendation**: Keep `-30dB`

### `speedup_factor`
- Current default: `2.0x`
- Research: 1.5x is sweet spot, 2x acceptable for deliberate speakers
- **Recommendation**: Keep `2.0x` default

### Loudnorm
- Current: `TP=-2.0` (conservative, good)
- YouTube target: `-14 LUFS`, True Peak `-1.5dBTP`
- **Recommendation**: Keep `TP=-2.0` (safer than -1.5 for unknown source quality)

---

## 📋 Pending Changes

### 1. `afftdn nr=15` → `nr=12`
- **File**: `src/editor.rs:393`
- **Change**: `afftdn=nr=15:tn=true` → `afftdn=nr=12:tn=true`

### 2. `duck_volume 0.2` → `0.15`
- **File**: `src/config.rs:283`
- **Change**: `default_duck_volume()` returns `0.15` instead of `0.2`

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