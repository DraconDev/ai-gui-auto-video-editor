#!/usr/bin/env bash
# Release helper: run tests, build, package tarball + checksum, tag, and publish.
#
# Usage:
#   ./scripts/release.sh              # release using version from Cargo.toml
#   ./scripts/release.sh 19.58.0      # release with explicit version
#   ./scripts/release.sh --cargo-publish  # also publish to crates.io
#
# The script will:
#   1. Run full test suite (cargo test + clippy)
#   2. Build release binary
#   3. Package tarball with binary, assets, docs, install script
#   4. Generate sha256 checksum
#   5. Create git tag (vVERSION)
#   6. Optionally publish to crates.io

set -euo pipefail

cd "$(dirname "$0")/.."

CARGO_PUBLISH=false
if [[ "${1:-}" == "--cargo-publish" ]]; then
    CARGO_PUBLISH=true
    shift
fi

# Get version from Cargo.toml if not specified
VERSION=${1:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}
if [ -z "$VERSION" ]; then
    echo "ERROR: Could not determine version from Cargo.toml"
    exit 1
fi

RELEASE_ROOT="release"
DIST_DIR="$RELEASE_ROOT/$VERSION"
ARCHIVE="$RELEASE_ROOT/agave-$VERSION.tar.gz"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[1;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { printf "${BLUE}> %s${NC}\n" "$1"; }
ok()   { printf "${GREEN}✓ %s${NC}\n" "$1"; }
warn() { printf "${YELLOW}! %s${NC}\n" "$1"; }
fail() { printf "${RED}✗ %s${NC}\n" "$1" >&2; exit 1; }

# ── Pre-flight ───────────────────────────────────────────────────────────────

info "Release version: $VERSION"

info "Checking for uncommitted changes..."
if ! git diff --quiet HEAD 2>/dev/null; then
    warn "Uncommitted changes detected. Commit or stash before releasing."
    git status --short
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]] || exit 1
fi

# ── Test ─────────────────────────────────────────────────────────────────────

info "Running full test suite..."
cargo test --all-features || fail "Tests failed"
cargo clippy --all-features -- -D warnings || fail "Clippy failed"
cargo fmt --all -- --check || fail "Format check failed"
ok "All checks pass"

# ── Build ─────────────────────────────────────────────────────────────────────

info "Building release binary..."
cargo build --release || fail "Build failed"
ok "Build complete"

# ── Package ──────────────────────────────────────────────────────────────────

info "Preparing release directory..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"/{bin,assets,docs}

info "Copying binary + assets..."
cp "target/release/agave" "$DIST_DIR/bin/"
cp assets/icon.svg assets/agave.desktop "$DIST_DIR/assets/"

info "Bundling docs..."
cp README.md CHANGELOG.md docs/customer-facing.md docs/release-locations.md "$DIST_DIR/docs/"
cp install.sh "$DIST_DIR/"

info "Creating archive $ARCHIVE..."
rm -f "$ARCHIVE"
tar -czf "$ARCHIVE" -C "$DIST_DIR" .

info "Generating checksum..."
sha256sum "$ARCHIVE" > "$RELEASE_ROOT/agave-$VERSION.sha256"

ok "Release $VERSION packaged in $RELEASE_ROOT/"

# ── Git tag ──────────────────────────────────────────────────────────────────

TAG="v$VERSION"
if git rev-parse "$TAG" >/dev/null 2>&1; then
    warn "Tag $TAG already exists. Skipping."
else
    info "Creating git tag $TAG..."
    git tag -a "$TAG" -m "Release $VERSION"
    ok "Tag $TAG created"
fi

# ── crates.io ────────────────────────────────────────────────────────────────

if [ "$CARGO_PUBLISH" = true ]; then
    info "Publishing to crates.io..."
    cargo publish
    ok "Published agave@$VERSION to crates.io"
else
    info "Skipping crates.io publish (pass --cargo-publish to enable)"
fi

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
printf "${GREEN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║              Release $VERSION Ready                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
printf "${NC}"
echo ""
echo "  Archive:  $ARCHIVE"
echo "  Checksum: $RELEASE_ROOT/agave-$VERSION.sha256"
echo "  Git tag:  $TAG"
echo ""
echo "Next steps:"
echo "  1. Push tag:          git push origin $TAG"
echo "  2. Create GitHub release with archive + checksum"
echo "  3. Publish to crates.io: ./scripts/release.sh --cargo-publish"
echo ""
