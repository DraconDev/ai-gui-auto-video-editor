# Full Code Audit Tasks

## Status: Pending

---

## Phase 1: Code Quality

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

## Phase 2: Audio Processing

### 2.1 Loudnorm
- [ ] Two-pass loudnorm working correctly
- [ ] JSON parse handles all cases (check `rfind('}')` fix)
- [ ] Fallback to single-pass if measurement fails
- [ ] Target LUFS matches platform (YouTube: -14, Podcast: -16)
- [ ] True peak ceiling appropriate (-2.0 conservative, -1.0 aggressive)

### 2.2 Noise Reduction
- [ ] `afftdn` settings appropriate for voice
- [ ] `nr` value not too aggressive (default 12)
- [ ] Noise floor (`nf`) not cutting harmonics

### 2.3 Ducking
- [ ] Background music ducked appropriately during speech
- [ ] Duck volume appropriate (-16 to -18dB typical)
- [ ] Fade in/out smooth (no clicks/pops)

### 2.4 Silence Detection
- [ ] Threshold appropriate for voice (-30dB standard)
- [ ] Min duration not too short (0.5s typical)
- [ ] Padding consistent (0.05s-0.2s based on content type)

### 2.5 Speedup Mode
- [ ] Off by default in all presets
- [ ] `speedup_factor` capped appropriately (2.0x max recommended)
- [ ] `min_silence_for_speedup` prevents tiny clips being sped up

---

## Phase 3: Video Processing

### 3.1 Trimming
- [ ] Chunked trimming to avoid FFmpeg arg limits
- [ ] Segments calculated correctly from silence detection
- [ ] Padding applied at cut points
- [ ] Keyframe alignment handled for stream copy

### 3.2 Scaling/Resolution
- [ ] Target resolution set correctly per preset
- [ ] Aspect ratio maintained (letterbox if needed)
- [ ] Vertical video handled for Shorts/Reels/TikTok

### 3.3 Stabilization
- [ ] Settings not too aggressive (shakiness=5, accuracy=15)
- [ ] Zoom not excessive (optzoom=1)
- [ ] Performance acceptable

### 3.4 Auto-Reframe (ML)
- [ ] Face detection confidence threshold appropriate
- [ ] Smoothed interpolation between keyframes
- [ ] Handles no-faces gracefully
- [ ] 9:16 crop centered on detected faces

### 3.5 Color Correction
- [ ] Box blur not applied to full frame (blur_background limitation noted)
- [ ] Color correction pass-through or minimal

---

## Phase 4: Config & Presets

### 4.1 Defaults
- [ ] All fields have sensible defaults
- [ ] Defaults match spec (see AGENTS.md)
- [ ] Serde defaults and Rust `Default` impl in sync

### 4.2 Merge Logic
- [ ] Scalar fields: only override if non-default
- [ ] Enum fields: always taken from explicit values
- [ ] Vec/Option fields: taken if present/non-empty
- [ ] CLI flags override everything
- [ ] Config file overrides presets

### 4.3 Presets
- [ ] YouTube: long-form, chapters, FCPXML export
- [ ] Shorts/TikTok/Reels: vertical, auto-reframe, captions/clips
- [ ] Podcast: -16 LUFS, subtitles, more padding
- [ ] Twitter: 2:20 max, landscape
- [ ] Minimal: just silence detection

### 4.4 Serialization
- [ ] `round_floats_in_value` prevents f32 artifacts
- [ ] TOML roundtrip works
- [ ] No losing precision on save/reload

---

## Phase 5: GUI

### 5.1 Theme
- [ ] Corner radius 0.0 (sharp edges)
- [ ] Red accent `rgb(230,57,70)`
- [ ] Dark background `rgb(14,14,16)`
- [ ] No hardcoded colors elsewhere

### 5.2 Navigation
- [ ] Sidebar shows all tabs (Dashboard, Queue, Settings)
- [ ] Folder panel in header
- [ ] Active state: red-tinted background + red border

### 5.3 Settings Panels
- [ ] Processing: silence mode, threshold, padding
- [ ] Audio: enhance, noise reduction, duck volume
- [ ] Video: resolution, quality, stabilization, reframe
- [ ] Exports: format options per tab
- [ ] Advanced: join mode, scene detection

### 5.4 Dashboard
- [ ] Activity log displays
- [ ] Toasts for notifications
- [ ] Stats summary accurate

### 5.5 Queue
- [ ] Shows pending files
- [ ] Progress updates live
- [ ] Can cancel/remove items

---

## Phase 6: Watch Mode & Shutdown

### 6.1 Watch Loop
- [ ] Polls folder at interval
- [ ] Detects new files
- [ ] Processes queue sequentially
- [ ] Bounded channel (1000) prevents memory growth

### 6.2 Graceful Shutdown
- [ ] `ctrlc` handler sets AtomicBool
- [ ] All watch loops check stop flag
- [ ] Multi-watch joins all threads on shutdown
- [ ] In-progress files handled (finish or abort)

---

## Phase 7: Integration Points

### 7.1 CLI
- [ ] All args parsed correctly
- [ ] --headless launches watch mode
- [ ] --gui explicit GUI launch
- [ ] --json output format works
- [ ] Dry-run shows duration estimates

### 7.2 Exports
- [ ] SRT subtitles from Whisper transcript
- [ ] ASS subtitles with styling
- [ ] FCPXML for Final Cut Pro
- [ ] EDL for NLEs
- [ ] Chapter markers (YouTube)

### 7.3 Highlight Clips
- [ ] Extract based on energy/speech
- [ ] `-ss` before `-i` for fast seeking
- [ ] `-c copy` for stream copy
- [ ] `-avoid_negative_ts make_zero`

### 7.4 Thumbnails
- [ ] Best frame extraction
- [ ] Frame at specific time
- [ ] Size/resolution options

---

## Phase 8: Dependencies & Performance

### 8.1 Dependencies
- [ ] No unused dependencies
- [ ] No known CVEs
- [ ] Version pins appropriate (not too loose)

### 8.2 Build
- [ ] `cargo build --release` works
- [ ] `cargo build --no-default-features --features cli` works (smaller binary)
- [ ] Build time reasonable
- [ ] Binary size acceptable

### 8.3 Tests
- [ ] Unit tests pass (548+)
- [ ] Integration tests pass (require ffmpeg)
- [ ] No flaky tests
- [ ] Coverage acceptable for critical paths

### 8.4 Clippy
- [ ] No warnings with deny-warnings
- [ ] No allow directives for real issues

---

## Phase 9: Documentation

### 9.1 Code Comments
- [ ] Complex logic explained
- [ ] FFmpeg filter strings documented
- [ ] Magic numbers defined

### 9.2 Spec Accuracy
- [ ] AGENTS.md matches implementation
- [ ] Known limitations current
- [ ] Bug fixes documented in changelog

### 9.3 Examples
- [ ] `ai-gui-auto-video-editor.example.toml` complete
- [ ] Preset TOML files accurate

---

## Phase 10: Edge Cases

### 10.1 Empty/Missing
- [ ] No input file handled gracefully
- [ ] No silence detected handled
- [ ] All silence handled (zero output)
- [ ] Corrupt file handled

### 10.2 Edge Values
- [ ] Very short video (1s)
- [ ] Very long video (10h)
- [ ] Very high silence ratio (>50%)
- [ ] No speech detected (transcription fails)

### 10.3 File System
- [ ] Output directory doesn't exist (created)
- [ ] Output file exists (overwritten with -y)
- [ ] Permission denied handled
- [ ] Disk full handled

### 10.4 Network
- [ ] Model download fails (retry, cache)
- [ ] HuggingFace rate limit handled

---

## Checklist Summary

```
Phase 1: Code Quality     [ ]/[ ]
Phase 2: Audio Processing  [ ]/[ ]
Phase 3: Video Processing  [ ]/[ ]
Phase 4: Config & Presets  [ ]/[ ]
Phase 5: GUI               [ ]/[ ]
Phase 6: Watch & Shutdown  [ ]/[ ]
Phase 7: Integration       [ ]/[ ]
Phase 8: Build & Tests     [ ]/[ ]
Phase 9: Documentation     [ ]/[ ]
Phase 10: Edge Cases       [ ]/[ ]
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
