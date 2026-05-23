# Full Code Audit Tasks

## Status: In Progress

---

## Phase 1: Code Quality ✅ DONE

### 1.1 Error Handling
- [x] No `.unwrap()` or `.expect()` in production code paths ✅
- [x] All fallible operations use `anyhow::Result<T>` with `?` ✅
- [x] Context added via `.context()` at crate boundaries ✅
- [x] No `panic!()` in production code ✅
- [x] `unimplemented!()` or `todo!()` checked (should be 0) ✅

### 1.2 Enum Exhaustiveness
- [x] All `match` statements handle all variants ✅
- [x] No wildcard `_` matches unless intentional ✅
- [x] Check: `SilenceMode`, `Preset`, `VideoResolution`, `JoinMode`, `WatermarkPosition`, `HwAccel` ✅

### 1.3 Safety
- [x] No `unsafe` blocks in production (FFT in stt_analyzer is OK, documented) ✅
- [x] No division by zero possibilities ✅
- [x] No out-of-bounds array access ✅
- [x] No integer overflow in release builds ✅

### 1.4 FFmpeg Security
- [x] All paths escaped for filter strings (`escape_ffmpeg_filter_path`) ✅
- [x] No direct string interpolation into shell commands ✅
- [x] Numeric args (scale, bitrate) validated ✅

---

## Phase 2: Audio Processing ✅ DONE

### 2.1 Loudnorm
- [x] Two-pass loudnorm working correctly ✅
- [x] JSON parse handles all cases (`rfind('}')` fix applied) ✅
- [x] Fallback to single-pass if measurement fails ✅
- [x] Target LUFS matches platform (YouTube: -14, Podcast: -16) ✅
- [x] True peak ceiling appropriate (`TP=-2.0` conservative) ✅

### 2.2 Noise Reduction
- [x] `afftdn` settings appropriate for voice ✅ (`nr=12`)
- [x] `nr` value not too aggressive (default 12) ✅

### 2.3 Ducking
- [x] Background music ducked appropriately during speech ✅
- [x] Duck volume appropriate (`0.15` = -16dBFS) ✅

### 2.4 Silence Detection
- [x] Threshold appropriate for voice (`-30dB` standard) ✅
- [x] Min duration not too short (`0.5s` typical) ✅
- [x] Padding consistent (`0.05s-0.2s` based on content type) ✅

### 2.5 Speedup Mode
- [x] Off by default in all presets ✅
- [x] `speedup_factor` capped appropriately (`2.0x` default) ✅
- [x] `min_silence_for_speedup` prevents tiny clips being sped up ✅

---

## Phase 3: Video Processing ✅ DONE

### 3.1 Trimming
- [x] Chunked trimming to avoid FFmpeg arg limits ✅
- [x] Segments calculated correctly from silence detection ✅
- [x] Padding applied at cut points ✅
- [x] Keyframe alignment handled for stream copy ✅

### 3.2 Scaling/Resolution
- [x] Target resolution set correctly per preset ✅
- [x] Aspect ratio maintained (letterbox if needed) ✅
- [x] Vertical video handled for Shorts/Reels/TikTok ✅

### 3.3 Stabilization
- [x] Settings not too aggressive (`shakiness=5, accuracy=15`) ✅
- [x] Zoom not excessive (`optzoom=1`) ✅

### 3.4 Auto-Reframe (ML)
- [x] Face detection confidence threshold appropriate (`0.5`) ✅
- [x] Smoothed interpolation between keyframes ✅
- [x] Handles no-faces gracefully ✅
- [x] 9:16 crop centered on detected faces ✅

### 3.5 Color Correction
- [x] Box blur not applied to full frame (limitation documented) ✅
- [x] Color correction pass-through or minimal ✅

---

## Phase 4: Config & Presets ✅ DONE

### 4.1 Defaults
- [x] All fields have sensible defaults ✅
- [x] Defaults match spec (see AGENTS.md) ✅
- [x] Serde defaults and Rust `Default` impl in sync ✅

### 4.2 Merge Logic
- [x] Scalar fields: only override if non-default ✅
- [x] Enum fields: always taken from explicit values ✅
- [x] Vec/Option fields: taken if present/non-empty ✅
- [x] CLI flags override everything ✅
- [x] Config file overrides presets ✅

### 4.3 Presets
- [x] YouTube: long-form, chapters, FCPXML export ✅
- [x] Shorts/TikTok/Reels: vertical, auto-reframe, captions/clips ✅
- [x] Podcast: -16 LUFS, subtitles, more padding ✅
- [x] Twitter: 2:20 max, landscape ✅
- [x] Minimal: just silence detection ✅

### 4.4 Serialization
- [x] `round_floats_in_value` prevents f32 artifacts ✅
- [x] TOML roundtrip works ✅

---

## Phase 5: GUI ✅ DONE

### 5.1 Theme
- [x] Corner radius 0.0 (sharp edges) ✅
- [x] Red accent `rgb(230,57,70)` ✅
- [x] Dark background `rgb(14,14,16)` ✅
- [x] No hardcoded colors elsewhere ✅

### 5.2 Navigation
- [x] Sidebar shows all tabs (Dashboard, Queue, Settings) ✅
- [x] Folder panel in header ✅
- [x] Active state: red-tinted background + red border ✅

### 5.3 Settings Panels
- [x] Processing: silence mode, threshold, padding ✅
- [x] Audio: enhance, noise reduction, duck volume ✅
- [x] Video: resolution, quality, stabilization, reframe ✅
- [x] Exports: format options per tab ✅
- [x] Advanced: join mode, scene detection ✅

### 5.4 Dashboard
- [x] Activity log displays ✅
- [x] Toasts for notifications ✅
- [x] Stats summary accurate ✅

### 5.5 Queue
- [x] Shows pending files ✅
- [x] Progress updates live ✅
- [x] Can cancel/remove items ✅

---

## Phase 6: Watch Mode & Shutdown ✅ DONE

### 6.1 Watch Loop
- [x] Polls folder at interval ✅
- [x] Detects new files ✅
- [x] Processes queue sequentially ✅
- [x] Bounded channel (1000) prevents memory growth ✅

### 6.2 Graceful Shutdown
- [x] `ctrlc` handler sets AtomicBool ✅
- [x] All watch loops check stop flag ✅
- [x] Multi-watch joins all threads on shutdown ✅
- [x] In-progress files handled (finish or abort) ✅

---

## Phase 7: Integration Points ✅ DONE

### 7.1 CLI
- [x] All args parsed correctly ✅
- [x] --headless launches watch mode ✅
- [x] --gui explicit GUI launch ✅
- [x] --json output format works ✅
- [x] Dry-run shows duration estimates ✅

### 7.2 Exports
- [x] SRT subtitles from Whisper transcript ✅
- [x] ASS subtitles with styling ✅
- [x] FCPXML for Final Cut Pro ✅
- [x] EDL for NLEs ✅
- [x] Chapter markers (YouTube) ✅

### 7.3 Highlight Clips
- [x] Extract based on energy/speech ✅
- [x] `-ss` before `-i` for fast seeking ✅
- [x] `-c copy` for stream copy ✅

---

## Phase 8: Build & Tests ✅ DONE

### 8.1 Build
- [x] `cargo build --release` works ✅
- [x] `cargo build --no-default-features --features cli` works ✅

### 8.2 Tests
- [x] Unit tests pass (548) ✅
- [x] Clippy clean (deny warnings) ✅

---

## Phase 9: Edge Cases ✅ DONE

### 9.1 Empty/Missing
- [x] No input file handled gracefully ✅
- [x] No silence detected handled ✅
- [x] Corrupt file handled ✅

### 9.2 Edge Values
- [x] Very short video (1s) - handled with min duration ✅
- [x] Very long video (10h) - chunked processing ✅
- [x] Very high silence ratio (>50%) - handled ✅

### 9.3 File System
- [x] Output directory doesn't exist (created) ✅
- [x] Output file exists (overwritten with -y) ✅

---

## Summary

```
Phase 1: Code Quality      ✅ COMPLETE
Phase 2: Audio Processing  ✅ COMPLETE
Phase 3: Video Processing   ✅ COMPLETE
Phase 4: Config & Presets  ✅ COMPLETE
Phase 5: GUI                ✅ COMPLETE
Phase 6: Watch & Shutdown   ✅ COMPLETE
Phase 7: Integration        ✅ COMPLETE
Phase 8: Build & Tests     ✅ COMPLETE
Phase 9: Edge Cases        ✅ COMPLETE
```

---

## Previous Fixes (do not regress)

- [x] progress.rs: SystemTime elapsed → duration_since + 5s tolerance
- [x] editor.rs: path truncation → {:016x} fixed-width hex
- [x] ml.rs: fps default → with_context() error propagation
- [x] editor.rs: loudnorm JSON parse → rfind('}') fix
- [x] main.rs: SilenceMode::Speedup → duration calculation
- [x] afftdn nr=15 → nr=12 (FFmpeg default)
- [x] duck_volume 0.2 → 0.15 (-16dBFS)
- [x] Silence speedup off by default (all presets use Cut)