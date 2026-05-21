# Autoresearch Ideas

## Promising Optimizations (Deferred)

### Build Performance
- [ ] Use `cargo check --lib` for faster incremental checks
- [ ] Evaluate `lto = true` impact on binary size
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

## Rejected Ideas
- (none yet)

## Notes
- Be cautious about premature optimization
- Focus on measurable improvements