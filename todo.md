# TODO — Full File Cleanup

## ✅ Remove stale release artifacts
- [x] Deleted `release/0.1.424/`, `release/0.1.467/`, `release/3.0.0/`, `release/13.2.0/`, `release/19.1.9/`, `release/19.2.2/`
- [x] Deleted all tarballs and checksums (`ai-vid-editor-0.1.424.*`, `0.1.467.*`, `3.0.0.*`, `13.2.0.*`, `19.1.9.*`, `19.2.2.*`)

## ✅ Update CHANGELOG.md
- [x] Added `[19.58.0]` section with all changes (bug fixes, code quality, features, GUI, cleanup)

## ✅ Update example config
- [x] `ai-vid-editor.example.toml` — added blur_background clarification comment
- [x] Added missing `[video]` fields: watermark, watermark_position, watermark_scale, blur_background, reframe, target_resolution
- [x] Added missing `speedup_factor` in `[silence]`

## ✅ Update docs with stale refs
- [x] `docs/superpowers/plans/2026-04-25-big-features.md` — `gui/tabs.rs` → `gui/tabs/mod.rs`
- [x] `docs/customer-facing.md` — added Ctrl+C graceful shutdown note

## ✅ Verify presets
- [x] All 4 presets (youtube, shorts, podcast, minimal) checked — no stale/renamed fields

## ✅ Check install.sh and scripts/release.sh
- [x] No references to old `tabs.rs` or old file structure

## Assets — Needs Manual Update
- [ ] `assets/Screenshot_20260319_124018.png` — outdated (shows old UI with rounded corners). Needs a new screenshot from the current build (zero-radius, red-highlight sidebar).

## ✅ Review other tracked files
- [x] `plans/project-specs.md` — no stale refs
- [x] `.github/dependabot.yml` — configured correctly
- [x] `.github/workflows/release.yml` — no stale refs
- [x] `CONTRIBUTING.md` — fine
- [x] `flake.nix` — no stale refs
- [x] `benches/parsers.rs` — compiles, relevant
- [x] `proptest-regressions/analyzer.txt` — normal, auto-managed
- [x] `.gitattributes` — managed by dracon-warden
- [x] `install.sh` — no stale refs
- [x] `scripts/release.sh` — no stale refs
- [x] `CHANGELOG.md` — historical `tabs.rs` refs are fine (they describe past versions)
