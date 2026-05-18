## Where to publish releases

This repo ships binaries and installers to end users. Make sure each release is published in all relevant channels so customers can download whichever package suits their platform.

1. **GitHub Releases**
   - Upload `release/agave-<version>.tar.gz` and the corresponding `.sha256` alongside release notes (highlight installer fixes + UI updates).
   - Pin the release and mark it as "Latest" until the next release is created.

2. **crates.io**
   - Published as `agave` on [crates.io](https://crates.io/crates/agave).
   - Users install with `cargo install agave`.
   - Run `cargo publish` (or `scripts/release.sh --cargo-publish`) to push new versions.
   - The `exclude` list in `Cargo.toml` keeps the package under the 10 MB upload limit.

3. **Website / Download page**
   - Mirror the GitHub artifact URL or host a static copy on your website.
   - Display checksums and quick instructions: `tar -xzf agave-<version>.tar.gz` and `./install.sh --user`.

4. **Package repositories**
   - Update any Linux package (e.g., Nixpkgs overlay, Arch AUR, Debian, Ubuntu PPA) with the same version/hash.
   - Publish AppImage/Deb packages that wrap the release binary + assets.

5. **Desktop stores** (optional)
   - Microsoft Store / Snap / Flatpak: rebuild around the release binary and include the same icon/desktop entry.

6. **Release notes & docs**
   - Update `README.md`, `docs/customer-facing.md`, and `docs/release-locations.md` with the final version number.
   - Document any breaking changes (new config keys, removed flags) so support teams can reference them.

7. **Support communications**
   - Email / Slack / forum posts: link to the release artifact, highlight install script improvements, and mention the onboarding wizard.

Keeping the instructions in this doc ensures every release reaches every channel and remains easy for customers to find and install.
