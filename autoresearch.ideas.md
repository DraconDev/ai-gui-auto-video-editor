# Autoresearch Ideas

## Completed Optimizations

### Binary Size Reduction (-13.5%)
```toml
[profile.release]
lto = true          # was "thin"
panic = "abort"     # eliminates unwinding
strip = true
opt-level = 2
codegen-units = 16  # faster compile
```
**Result**: 41,235 KB → 35,655 KB

## Promising Optimizations (Deferred)

### Build Performance
- [ ] Use `cargo check --lib` for faster incremental checks (minor benefit)
- [ ] Consider sccache for caching compilation results

### Runtime Performance
- [ ] Consider `SmallVec<[T; N]>` for small collections (151 string formats found)
- [ ] Profile batch processor thread pool sizing
- [ ] FFmpeg command builder pattern (40 Command::new calls)
- [ ] Buffer reuse for string building

### Memory Efficiency
- [ ] Profile temp file allocation patterns
- [ ] Evaluate `Arc<str>` vs `String` for shared strings

## Audit Findings

### Build Performance
- **Clean incremental build**: 6.81s for `cargo check`
- **Full release build**: 247s (dominated by ML deps)
- **Dev build**: 54s cold, 1.15s incremental

### Binary Size Contributors (after optimization)
1. tract-onnx (~10MB+)
2. candle-core/candle-nn (~5MB+)
3. eframe/egui (~5MB+)
4. image crate (~2MB+)
5. tokenizers (~2MB+)

### Code Quality
- 0 clippy warnings
- 552 tests passing
- Proper error handling with anyhow
- Thread-safe patterns well-implemented

## Rejected Ideas
- **Remove ML features**: Core functionality
- **Remove GUI**: User-facing requirement
- **opt-level = 3**: Diminishing returns, much slower compile
- **lto = "fat"**: Marginal gain, much longer compile time

## Notes
- Full audit complete - codebase is production quality
- Binary optimization complete
- All 552 tests passing
- 0 clippy warnings