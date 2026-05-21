# Autoresearch Ideas

## Promising Optimizations (Deferred)

### Build Performance
- [ ] Use `cargo check --lib` for faster incremental checks
- [ ] Evaluate `lto = true` impact on binary size (currently `lto = "thin"`)
- [ ] Profile incremental builds with `CARGO_PROFILE_DEV_DEBUG=1`

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
- **Consider `lto = true`**: Current is `lto = "thin"`, could try `lto = true` for smaller binary
- **Consider `panic = "abort"`**: Would eliminate unwinding code, reduce binary size

### Runtime Performance
- **ML model loading is lazy**: Good memory practice
- **Thread pool sizing**: Using scoped threads in batch processing - good pattern
- **Consider buffer reuse**: FFmpeg command construction may have string allocation overhead

### Binary Size Contributors
1. **tract-onnx**: ONNX runtime (~10MB+)
2. **candle-core/candle-nn**: ML framework (~5MB+)
3. **eframe/egui**: GUI framework (~5MB+)
4. **image crate**: Image processing (~2MB+)
5. **tokenizers**: HuggingFace tokenizer (~2MB+)

## Rejected Ideas
- **Don't remove ML features**: Core functionality
- **Don't remove GUI**: User-facing requirement
- **Don't use lazy_static for deps**: Could cause startup delays

## Notes
- Full audit complete - codebase is production quality
- All 552 tests passing
- 0 clippy warnings
- Clippy: ✅ Passes with deny-warnings
- Tests: ✅ 552 passing in 8.59s