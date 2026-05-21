# Autoresearch Ideas

## Completed Optimizations

### Binary Size Reduction (-13.5%)
```toml
[profile.release]
lto = true          # was "thin" - better cross-crate inlining
panic = "abort"     # eliminates unwinding code
strip = true
opt-level = 2       # good balance
codegen-units = 16  # parallel codegen
```
**Result**: 41,235 KB → 35,655 KB

## Audit Complete - No Further Action Required

### Code Quality
- ✅ 552 tests passing (551 unit + 1 integration)
- ✅ 0 clippy warnings
- ✅ Idiomatic Rust error handling with anyhow
- ✅ Thread-safe patterns correctly implemented

### Build Performance
- ✅ `cargo check --all-features`: 0.6s (clippy), 6.8s (full)
- ✅ `cargo test --lib --test-threads=4`: ~6-13s
- ⚠️ Release builds: ~247s (dominated by ML deps - acceptable)

### Binary Size
- ✅ Optimized: 35,655 KB (down from 41,235 KB)
- ⚠️ Dominated by tract-onnx, candle-*, eframe/egui dependencies

### Deferred Ideas (Low Priority)
- SmallVec for string collections (minor benefit)
- FFmpeg command builder pattern (40 calls - refactoring effort vs benefit)
- sccache for faster rebuilds (CI/incremental only)

## What We Did NOT Change
- ❌ ML features (tract-onnx, candle) - core functionality
- ❌ GUI (eframe/egui) - user-facing requirement
- ❌ opt-level = 3 - diminishing returns
- ❌ lto = "fat" - marginal gain, much longer compile

## Final Metrics
| Metric | Value |
|--------|-------|
| Binary Size | 35,655 KB (-13.5%) |
| Tests | 552 passing |
| Clippy | 0 warnings |
| Test Time (parallel) | 6-14s |