# Code Review TODO — AI Video Editor

## 🔴 CRITICAL

### 1. `progress.rs:44,65` — `SystemTime::elapsed()` used as age-stamp (LOGIC BUG)
- [ ] `is_completed()` and `mark_completed()` store file *age* (elapsed time) instead of absolute mtime
- [ ] Comparison `saved_mtime.abs_diff(current_mtime)` fails when time passes between mark and check
- **Fix**: Store raw `SystemTime` from `modified()`, compare with `duration_since()`

### 2. `editor.rs:648-656` — Join path can exceed `NAME_MAX`
- [ ] `{:x}` on u128 produces up to 32 hex chars + prefix, risks exceeding filesystem limits
- **Fix**: Use `{:016x}` (fixed width) or hash the nanoseconds

### 3. `ml.rs:121-125` — Silent FPS default hides parse failures
- [ ] `unwrap_or(25.0)` silently returns wrong fps on malformed ffprobe output
- **Fix**: Propagate errors with `.context()` — downstream ops produce wrong results silently

---

## 🟠 MEDIUM

### 4. `gui.rs:416-445` — `activity_log` grows unbounded
- [ ] `activity_log` has no cap; long-running sessions consume unbounded RAM
- **Fix**: Cap at e.g. 1000 entries, trim oldest

### 5. `hwaccel.rs:264,273` — `panic!` in tests (style)
- [ ] Use `assert_eq!(default, HwAccel::None)` instead of match+panic
- **Fix**: Replace panic with proper assertion

### 6. `config.rs:merge()` — No compile-time enforcement for new fields
- [ ] New fields on config structs silently bypass merge logic
- **Fix**: Consider derive macro or at minimum add a test that compares merge behavior for all fields

### 7. Dead code cleanup — 12 `#[allow(dead_code)]` across 5 files
- [ ] `src/gui.rs:98,109,262,428,433,474,664` — unused sidebar items, SettingsCategory methods
- [ ] `src/gui/theme.rs:125,178` — unused style functions
- [ ] `src/gui/processing.rs:261,408` — test helpers
- [ ] `src/batch_processor.rs:1632` — test mock
- **Fix**: Remove or gate behind feature flags

### 8. `batch_processor.rs:700-715` — ANSI escapes in format string
- [ ] `\x1b[32m` etc. produces garbage if output redirected to file
- **Fix**: Detect non-TTY, strip ANSI or use a terminal color crate

---

## 🟡 LOW / OBSERVATIONS

### 9. `stt_analyzer.rs:600` — `partial_cmp.unwrap()` in test
- [ ] Works but fragile if NaN introduced
- **Fix**: Use `total_cmp()` or `unwrap_or_else`

### 10. `watch.rs:95` — `thread::sleep` in watch loop
- [ ] Sleep blocks thread, but stop flag is checked after each sleep — acceptable
- **Observation**: Could use `park_timeout` for cleaner shutdown

### 11. `ml.rs` — Duplicated dimension-fetching logic
- [ ] `FrameExtractor::get_video_dimensions` duplicates `FfmpegDurationGetter` pattern
- **Fix**: Extract to shared utility or use `FfmpegDurationGetter`-like trait

### 12. `batch_processor.rs:126,137` — CachingDurationGetter two-phase locking
- [ ] Correct pattern but cache could miss concurrent requests
- **Observation**: Not a bug but worth reviewing under high-concurrency loads

---

## ✅ DONE WELL (leave as-is)

- [x] FFmpeg escaping via `escape_ffmpeg_filter_path()` consistently used
- [x] Mutex poisoning handled with `unwrap_or_else(|p| p.into_inner())`
- [x] Bounded channels `sync_channel(1000)` prevent memory growth
- [x] AtomicBool stop flags with `Ordering::SeqCst`
- [x] RAII temp cleanup with `TempFileGuard`, `TempDir`, `TempFile`
- [x] Config priority: CLI > file > preset > defaults
- [x] Atomic rename pattern for model caching
- [x] Error provenance with `.context()`