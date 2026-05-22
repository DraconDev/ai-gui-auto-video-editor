# Code Review — Full Audit (2026-05-22)

## Stats
- **Source**: 27 files, ~21,600 lines (`src/`)
- **Tests**: 552 passing, 0 failing
- **Build**: clean, clippy clean (`-D warnings`)

---

## 🔴 CRITICAL — FIXED in prior session

- [x] `progress.rs:44,65` — `SystemTime::elapsed()` logic bug → fixed with raw mtime comparison
- [x] `editor.rs:648` — `{:x}` path truncation → fixed with `{:016x}`
- [x] `ml.rs:121-125` — silent `unwrap_or(25.0)` fps default → fixed with error propagation

---

## 🟠 MEDIUM — FIXED in prior session

- [x] `hwaccel.rs:264,273` — `panic!` in tests → `assert_eq!`
- [x] `gui.rs:416-445` — unbounded `activity_log` → already capped at 500
- [x] `batch_processor.rs:700-715` — ANSI escapes always on → TTY detection added
- [x] `stt_analyzer.rs:600` — `partial_cmp.unwrap()` → `to_bits().cmp()`
- [x] Dead code removed (9 of 12 `#[allow(dead_code)]` items)

---

## 🟡 CLIPPY / WARNINGS (this audit)

- [x] `editor.rs:652` — `.min(u128::MAX)` on `as_nanos()` is never > MAX → **removed** (clippy error)

---

## ⚪ PRODUCTION CODE ANALYSIS

### `.unwrap()` / `.expect()` in production (non-test) code

**Result: 0 panics in production paths.**

Every `.unwrap()` found outside `#[cfg(test)]`/`mod tests` is either:
- A `tempfile::tempdir()` in test-only wrappers (watermark, thumbnail, stt_analyzer, etc.)
- An `fs::write()` call inside a `#[test]` module
- A `try_parse_from(...).unwrap()` in `main.rs` tests only

### `.expect("Index 1")` / `.expect("Index 2")` in `exporter.rs`

Located at lines 1038 and 1044 — inside `#[test] fn test_export_srt_sorted_by_start_time()`.
Searches for substring `"1\n"` and `"2\n"` in generated SRT output.
**Verdict: test-only, safe.**

### `panic!` in production

**0 panics in production code.**  
The 2 `panic!` calls (previously fixed in `hwaccel.rs`) were in `#[test]` functions.

### `unsafe` blocks

**1 block, documented, safe:**

`src/stt_analyzer.rs:81` — memory-mapped safetensor weight files.
Safety invariant: file length verified before mmap, file descriptor held through use.

---

## 📋 OPEN (deferred / low priority)

1. **`config.rs:merge()`** — no compile-time enforcement that new fields are included
   - Low risk: defaults-only merge, low churn on struct fields
   - Could use derive macro or test comparison

2. **Duplicated dimension-fetching** in `ml.rs`
   - Low priority: cosmetic, not a bug

3. **3 remaining `#[allow(dead_code)]`** — reserved for future use:
   - `QueueEvent` (cross-module, queue worker integration)
   - `duplicate_folder` (GUI folder management)
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