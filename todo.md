# Code Review — Full Audit (2026-05-22/23)

## Stats
- **Source**: 27 files, ~21,600 lines (`src/`)
- **Tests**: 552 passing, 0 failing
- **Build**: clean, clippy clean (`-D warnings`)

---

## 🔴 CRITICAL — FIXED

- [x] `progress.rs:44,65` — `SystemTime::elapsed()` logic bug → raw mtime comparison with 5s tolerance
- [x] `editor.rs:648` — `{:x}` path truncation → `{:016x}` fixed-width hex
- [x] `ml.rs:121-125` — silent `unwrap_or(25.0)` fps default → error propagation with `.context()`
- [x] `editor.rs:744` — **loudnorm JSON parse bug** (NEW) → `rfind('}')` instead of `find('}')`
  - FFmpeg outputs `"normalization_type" : "dynamic"` where `"dynamic"` contains a `}`
  - `find('}')` found the `}` inside the string value first → truncated JSON → missing `offset`
  - Missing `offset` → FFmpeg used `offset=0.0` → over-corrected gain → loud/distorted audio

---

## 🟠 MEDIUM — FIXED

- [x] `hwaccel.rs:264,273` — `panic!` in tests → `assert_eq!`
- [x] `gui.rs:416-445` — unbounded `activity_log` → already capped at 500
- [x] `batch_processor.rs:700-715` — ANSI escapes always on → `std::io::IsTerminal` TTY detection
- [x] `stt_analyzer.rs:600` — `partial_cmp.unwrap()` → `to_bits().cmp()`
- [x] Dead code removed (9 of 12 `#[allow(dead_code)]` items removed)

---

## 🟡 CLIPPY / WARNINGS — FIXED

- [x] `editor.rs:652` — `.min(u128::MAX)` on `as_nanos()` is never > MAX → **removed**
- [x] `batch_processor.rs:2169` — `assert!(result.len() >= 0)` useless comparison → removed

---

## ⚪ PRODUCTION CODE ANALYSIS

### `.unwrap()` / `.expect()` in production (non-test) code

**Result: 0 panics in production paths.**

Every `.unwrap()` outside `#[cfg(test)]`/`mod tests` is either:
- `tempfile::tempdir()` in test-only wrappers (always succeeds)
- `fs::write()` in `#[test]` modules (controlled test temp dirs)
- `Cli::try_parse_from(...).unwrap()` in `main.rs` tests only

### `.expect("Index 1/2")` in `exporter.rs`

Lines 1038/1044 — inside `#[test] fn test_export_srt_sorted_by_start_time()`.
Searches for `"1\n"` and `"2\n"` in generated SRT output. **Test-only, safe.**

### `panic!` in production

**0 panics.** 2 previous `panic!` calls (hwaccel.rs) were in `#[test]` functions and fixed.

### `unsafe` blocks

**1 block, documented, safe:**
`src/stt_analyzer.rs:81` — mmap of safetensor weight files. File length verified before mmap; fd held through use.

---

## 📋 OPEN / DEFERRED

1. **`config.rs:merge()`** — no compile-time enforcement for new fields
   - Low risk: defaults-only merge, low churn
2. **Duplicated dimension-fetching** in `ml.rs` — cosmetic, not a bug
3. **3 remaining `#[allow(dead_code)]`** — reserved for future use:
   - `QueueEvent` (cross-module queue worker)
   - `duplicate_folder` (future folder management)
   - `make_test_folder_state` / `build_folder_config` (integration test helpers)

---

## ✅ STATUS

| Check | Result |
|---|---|
| Production `.unwrap()` panics | 0 |
| Production `panic!` | 0 |
| `unsafe` blocks | 1 (safe, documented) |
| Clippy errors | 0 |
| Clippy warnings | 0 |
| Tests | 552/552 pass |
| Build | clean |

---

## 🗒 AUDIO ENHANCER NOTE (off-audit, 2026-05-23)

`enhance_audio()` applies two-pass loudnorm + EQ. The EQ boost at 4kHz (`g=1.5`) can add
harshness/sibilance. Professional default should probably be:
```
highpass=f=80,loudnorm=I={target}:TP=-1.5:LRA=11:linear=true
```
(no EQ, higher LRA, -1.5dB true peak). Deferred — audio processing is a user preference
area and the fix is low priority.