# Audio/Video Options Audit

## Status: In Progress

---

## Audio

### ✅ `enhance_audio` (loudnorm)
- **Status**: Fixed
- **File**: `src/editor.rs:330`
- **Defaults**: highpass=60, TP=-2.0, LRA=7, no EQ
- **Target**: -14 LUFS (YouTube)
- **Bug fixed**: JSON parse truncation (`rfind('}')` instead of `find('}')`) — was causing distorted audio

### ✅ Silence Speedup
- **Status**: Fixed — off by default across all presets
- **Shorts, TikTok, Reels**: now use `SilenceMode::Cut` instead of `Speedup`
- **GUI**: "Speed Up" option available in dropdown for users who want it
- **Default speedup_factor**: 2.0x (safer than previous 4.0x)
- **Fields**: `speedup_factor` and `min_silence_for_speedup` added to `SilenceConfig`

### 🔍 `noise_reduction` (afftdn)
- **File**: `src/editor.rs:393`
- **Current**: `afftdn=nr=15:tn=true`
- **TODO**: Is `nr=15` too aggressive? FFmpeg default is 12.
- **TODO**: Check if `tn=true` is worth the slowdown

### 🔍 `duck_volume`
- **File**: `src/config.rs:283`
- **Default**: 0.2 (-14dB during speech)
- **TODO**: Is 0.2 appropriate? Could be too quiet or too loud.

### 🔍 Silence detection thresholds
- **Files**: `src/config.rs`, `src/analyzer.rs`
- **Params**: threshold_db=-30, padding=0.1s, min_duration=0.5s
- **TODO**: Audit for reasonableness (all seem fine)

---

## Video

### 🔍 Stabilization
- **File**: `src/editor.rs:429`
- **Current**: `shakiness=5, accuracy=15, smoothing=10, optzoom=1, interpol=bicubic`
- **TODO**: Settings seem reasonable, low priority

### 🔍 Video quality
- **File**: `src/editor.rs:run_trim_filter_job`
- **Current**: libx264 CRF=20, NVENC cq=23, AAC 192kbps 48kHz
- **TODO**: Are these CRF values appropriate? Seems fine.

### 🔍 Auto-reframe (ML)
- **File**: `src/ml.rs`
- **TODO**: Check face detection confidence threshold
- **TODO**: Check smoothing/interpolation settings

### 🔍 Scene detection
- **File**: `src/scene_detection.rs`, `src/config.rs`
- **TODO**: Check threshold defaults (currently 0.10)

---

## Priority Order
1. [x] ~~`enhance_audio`~~ — done
2. [x] ~~Silence speedup off by default~~ — done
3. [ ] `noise_reduction` (afftdn) — next
4. [ ] `duck_volume`
5. [ ] Silence detection thresholds
6. [ ] Auto-reframe settings
7. [ ] All others (lower impact)