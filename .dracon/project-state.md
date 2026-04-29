# Project State

## Current Focus
Update release script to build the project with default features instead of explicitly enabling the `gui` feature flag.

## Completed
- [x] Release script now uses `cargo build --release` to build the binary with default feature set, removing the explicit `--features gui` flag.
