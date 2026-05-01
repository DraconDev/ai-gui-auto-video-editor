# Project State

## Current Focus
Add conditional compilation for CLI feature in `main.rs`

## Context
The change enables the CLI feature to be optionally compiled using the `#[cfg(feature = "cli")]` attribute, which is a common pattern for feature flags in Rust projects.

## Completed
- [x] Added `#[cfg(feature = "cli")]` attribute to `use clap::Parser`
- [x] Added `#[cfg_attr(feature = "cli", derive(clap::Parser))]` to the `Cli` struct

## In Progress
- [ ] None

## Blockers
- None

## Next Steps
1. Verify the feature flag works as expected in build configurations
2. Update documentation to reflect the new feature flag usage
