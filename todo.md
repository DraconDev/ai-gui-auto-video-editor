# Audio/Video Options Audit

## Status: TODO

---

## Audio Options

### ✅ `enhance_audio` (loudnorm)
- **Status**: Fixed (bug + defaults tuned)
- **File**: `src/editor.rs:330`
- **Defaults**: highpass=60, TP=-2.0, LRA=7, no EQ
- **Target**: -14 LUFS (YouTube)
- **Bugs found**: JSON parse truncation (`rfind` fix)

### 🔍 `noise_reduction` (afftdn)
- **File**: `src/editor.rs:393`
- **Current**: `afftdn=nr=15:tn=true`
- **TODO**: Audit settings — is `nr=15` too aggressive? Too conservative?
- **TODO**: Check if `tn=true` is worth the slowdown
- **TODO**: Check if `nf` param is worth adding for heavy noise

### 🔍 `target_lufs`
- **File**: `src/config.rs:280`
- **Default**: -14.0 (YouTube)
- **TODO**: Confirm -14 is correct default vs -16 (podcast)

### 🔍 `duck_volume`
- **File**: `src/config.rs`
- **Default**: 0.2
- **TODO**: Is 0.2 (-14dB) appropriate? Too loud/quiet?

### 🔍 Silence detection
- **Files**: `src/config.rs`, `src/analyzer.rs`
- **Params**: threshold_db, padding, min_duration
- **TODO**: Audit all silence thresholds for reasonableness

### 🔍 Audio speedup (silence removal)
- **File**: `src/config.rs`
- **Params**: speedup_factor, min_silence_for_speedup
- **TODO**: Audit speedup defaults — is 1.5x too aggressive?

---

## Video Options

### 🔍 Stabilization
- **File**: `src/editor.rs:426`
- **TODO**: Check vidstab settings — shakiness, accuracy, smoothing

### 🔍 Color correction
- **File**: `src/editor.rs`
- **TODO**: Audit eq/curves/gamma settings

### 🔍 Scene detection
- **File**: `src/scene_detection.rs`, `src/config.rs`
- **TODO**: Check threshold defaults

### 🔍 Auto-reframe (ML)
- **File**: `src/ml.rs`
- **TODO**: Check face detection confidence threshold
- **TODO**: Check smoothing/interpolation settings

---

## Export Options

### 🔍 Resolution presets
- **File**: `src/config.rs`
- **TODO**: Audit resolution options (YouTube, Shorts, Podcast)

### 🔍 Quality/bitrate settings
- **File**: `src/editor.rs:run_trim_filter_job`
- **Current**: libx264 CRF=20, NVENC cq=23, AAC 192kbps
- **TODO**: Are these CRF values appropriate?

### 🔍 Hardware acceleration
- **File**: `src/hwaccel.rs`
- **TODO**: Check default codec priority

---

## Priority Order
1. `noise_reduction` (afftdn) — most impactful after loudnorm
2. `duck_volume` — noticeable quality impact
3. Silence detection thresholds — common user complaint area
4. Speedup factor defaults
5. Stabilization settings
6. All others (lower impact)