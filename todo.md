# TODO — Full File Cleanup

## Stale Release Artifacts — Consider Removing
These are old release bundles tracked in git. Current version is 19.57.1; these are from 0.1.x, 3.0.0, 13.2.0, 19.1.x, 19.2.x — all outdated.

- [ ] `release/0.1.424/` — entire directory (ancient)
- [ ] `release/0.1.467/` — entire directory (ancient)
- [ ] `release/3.0.0/` — entire directory (outdated)
- [ ] `release/13.2.0/` — entire directory (outdated)
- [ ] `release/ai-vid-editor-0.1.424.tar.gz` — ancient tarball
- [ ] `release/ai-vid-editor-0.1.424.sha256` — ancient checksum
- [ ] `release/ai-vid-editor-0.1.467.tar.gz` — ancient tarball
- [ ] `release/ai-vid-editor-0.1.467.sha256` — ancient checksum
- [ ] `release/ai-vid-editor-3.0.0.tar.gz` — outdated tarball
- [ ] `release/ai-vid-editor-3.0.0.sha256` — outdated checksum
- [ ] `release/ai-vid-editor-13.2.0.tar.gz` — outdated tarball
- [ ] `release/ai-vid-editor-13.2.0.sha256` — outdated checksum
- [ ] `release/ai-vid-editor-19.1.9.sha256` — orphaned checksum (no matching tarball)
- [ ] `release/ai-vid-editor-19.2.2.tar.gz` — outdated tarball
- [ ] `release/ai-vid-editor-19.2.2.sha256` — outdated checksum

## CHANGELOG.md — Update for 19.57.1
- [ ] Add `[19.57.1]` section documenting all changes from this session:
  - P0: TempDir collision fix, create_test_video dedup, blur_background doc fix, VideoConfig::default() bug
  - P1: VideoResolution::parse_name(), fix_floats → round_floats_in_value, bounded sync_channel
  - P2: clip extraction keyframe-seeking, FolderSettings::is_default() simplification
  - P3: signal handling (ctrlc), gui/tabs.rs split, corner radius → 0, sidebar highlight fix

## Example Config — Update `ai-vid-editor.example.toml`
- [ ] `blur_background = false` in watch_folders section — add comment that this applies uniform boxblur, not ML person segmentation
- [ ] Missing `watermark` / `watermark_position` / `watermark_scale` fields in `[video]` section
- [ ] Missing `speedup_factor` in `[silence]` section (documented in code but not in example)

## Docs — Update References
- [ ] `CHANGELOG.md:135` — `tabs.rs` reference is historical (fine, but add note it's now `tabs/`)
- [ ] `CHANGELOG.md:207` — same, historical `tabs.rs` reference
- [ ] `docs/superpowers/plans/2026-04-25-big-features.md` — still references `gui/tabs.rs` in Tasks 3 and 4; update to `gui/tabs/`
- [ ] `docs/customer-facing.md` — no stale refs, but could mention Ctrl+C graceful shutdown
- [ ] `docs/linux-release-guide.md` — no stale refs, fine
- [ ] `docs/release-locations.md` — no stale refs, fine

## Release Docs — Old Versions Have Stale Content
- [ ] `release/13.2.0/docs/CHANGELOG.md` — references `tabs.rs`
- [ ] `release/3.0.0/docs/` — likely outdated
- [ ] `release/0.1.424/docs/` — definitely outdated
- [ ] `release/0.1.467/docs/` — definitely outdated
- Decision: Either delete old release dirs entirely (see above), or leave as-is since they're snapshots

## Presets — Check for Stale Fields
- [ ] `presets/youtube.toml` — verify no references to removed/renamed fields
- [ ] `presets/shorts.toml` — same
- [ ] `presets/podcast.toml` — same
- [ ] `presets/minimal.toml` — same

## Benchmarks
- [ ] `benches/parsers.rs` — verify this still compiles and is relevant

## Proptest Regressions
- [ ] `proptest-regressions/analyzer.txt` — check if stale; proptest creates these automatically

## Other Tracked Files to Review
- [ ] `plans/project-specs.md` — may be outdated
- [ ] `.github/dependabot.yml` — verify it's configured correctly
- [ ] `.github/workflows/cla.yml` — CLA assistant workflow, fine
- [ ] `.github/workflows/release.yml` — verify references current version scheme
- [ ] `COMMERCIAL-LICENSE.md` — fine if intentional dual-licensing
- [ ] `CONTRIBUTING.md` — check for stale instructions
- [ ] `CLA.md` — fine if needed for project
- [ ] `flake.nix` — Nix build support, verify it still works
- [ ] `.gitattributes` — check contents
- [ ] `install.sh` — check references to old file structure
- [ ] `scripts/release.sh` — check references to old file structure (e.g. tabs.rs)

## Assets
- [ ] `assets/Screenshot_20260319_124018.png` — README references this; consider updating with current UI (zero-radius, red-highlight sidebar)
- [ ] `assets/ai-vid-editor.desktop` — verify desktop entry is current
- [ ] `assets/icon.svg` — fine
- [ ] `assets/DejaVuSansSymbols.ttf` — embedded font, fine
- [ ] `assets/NotoEmojiSubset.ttf` — embedded emoji font, fine
