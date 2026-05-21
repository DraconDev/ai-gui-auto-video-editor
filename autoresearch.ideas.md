# Autoresearch Ideas

## Promising Optimizations (Deferred)

### Build Performance
- [x] Evaluate `lto = true` impact on binary size (DONE: saved 13.5%)
- [x] Evaluate `panic = "abort"` impact (DONE: combined with lto)
- [ ] Profile incremental builds with `CARGO_PROFILE_DEV_DEBUG=1`
- [ ] Use `cargo check --lib` for faster incremental checks

### Runtime Performance
- [ ] Consider `SmallVec<[T; N]>` for small collections
- [ ] Profile batch processor thread pool sizing
- [ ] Analyze FFmpeg command construction overhead
- [ ] Consider buffer reuse for string building

### Memory Efficiency
- [ ] Profile temp file allocation patterns
- [ ] Evaluate `Arc<str>` vs `String` for shared strings
- [ ] Consider `bytes` crate for binary data

### Code Quality
- [ ] Extract common FFmpeg argument patterns
- [ ] Consider builder pattern for complex commands
- [ ] Add benchmarks for hot paths

## Deferred - Audit Complete

### Build Performance
- **Incremental builds are fast**: 1.15s for `cargo check --all-features`
- **Cold release builds are slow**: 247s due to heavy ML dependencies (candle-core, candle-nn, tract-onnx)

### Binary Size Contributors
1. **tract-onnx**: ONNX runtime (~10MB+)
2. **candle-core/candle-nn**: ML framework (~5MB+)
3. **eframe/egui**: GUI framework (~5MB+)
4. **image crate**: Image processing (~2MB+)
5. **tokenizers**: HuggingFace tokenizer (~2MB+)

## Applied Optimizations (Keep)

### Profile Release Optimizations
```toml
[profile.release]
lto = true          # was "thin" - saves ~10%
panic = "abort"     # eliminates unwinding code - saves ~3%
strip = true        # removes debug symbols
opt-level = 2       # good balance of speed/size
codegen-units = 16  # faster compile (parallel codegen)
```

**Result**: Binary reduced from 41,235 KB → 35,655 KB (-13.5%)

## Rejected Ideas
- **Don't remove ML features**: Core functionality
- **Don't remove GUI**: User-facing requirement
- **Don't use lazy_static for deps**: Could cause startup delays
- **Don't use opt-level = 3**: Diminishing returns, much slower compile

## Notes
- Full audit complete - codebase is production quality
- All 552 tests passing
- 0 clippy warnings
- Clippy: ✅ Passes with deny-warnings
- Tests: ✅ 552 passing in 8.59s (dev) to 13.56s (with ML features)