# AI Video Editor - Common Commands

# Run the GUI
gui:
    cargo run -- --gui

# Run headless (watch mode for configured folders)
watch:
    cargo run -- --headless

# Run CLI with a file
cli input output:
    cargo run --release -- -i {{input}} -o {{output}}

# Build release (includes both CLI and GUI)
build:
    cargo build --release

# Build CLI only (smaller binary)
build-cli:
    cargo build --release --no-default-features --features cli

# Quick install: build & replace binary (kills running instance first)
install-quick:
    ./install.sh --user --quick

# Full user install (binary + icon + desktop entry + config)
install:
    ./install.sh --user

# Uninstall
uninstall:
    ./install.sh --uninstall

# Run tests (unit tests only, no ffmpeg required)
test:
    cargo test --lib

# Run full test suite (requires ffmpeg)
test-full:
    cargo test --all-features

# Generate default config
config:
    cargo run --release -- --generate-config > ai-gui-auto-video-editor.toml

# Watch a folder
watch input output:
    cargo run --release -- --watch {{input}} -O {{output}}

# Dry run (preview without processing)
dry input:
    cargo run --release -- -i {{input}} --dry-run

# Clean build artifacts
clean:
    cargo clean

# Check for issues
check:
    cargo check --all-features

# Format code
fmt:
    cargo fmt

# Lint
lint:
    cargo clippy --all-features -- -D warnings
    cargo fmt --all -- --check

# Full CI check (lint + test + check all feature combos)
ci: lint test-full
    cargo check --all-features
    cargo check --no-default-features --features cli
    cargo check --no-default-features --features gui

# Audit dependencies for vulnerabilities
audit:
    cargo audit 2>/dev/null || cargo install cargo-audit && cargo audit

# Check for outdated dependencies
outdated:
    cargo outdated 2>/dev/null || cargo install cargo-outdated && cargo outdated
