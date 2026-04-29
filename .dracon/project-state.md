# Project State

## Current Focus
Refactor name‑parsing APIs usage and improve code readability.

## Completed
- [x] Refactor long CLI help string in `main.rs` into a multi‑line `println!` for better readability.
- [x] Update GUI processing tests to use `Preset::parse_name` instead of the removed/renamed `Preset::from_str`.
- [x] Update pipeline integration test to use `HwAccel::parse_name` for round‑trip name parsing validation.
