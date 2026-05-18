# AGAVE - AI GUI Auto Video Editor

[![crates.io](https://img.shields.io/crates/v/ai-gui-auto-video-editor.svg)](https://crates.io/crates/ai-gui-auto-video-editor) [![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/DraconDev/ai-gui-auto-video-editor?label=)](https://github.com/DraconDev/ai-gui-auto-video-editor/releases) [![CI](https://github.com/DraconDev/ai-gui-auto-video-editor/actions/workflows/ci.yml/badge.svg)](https://github.com/DraconDev/ai-gui-auto-video-editor/actions/workflows/ci.yml) [![License](https://img.shields.io/crates/l/ai-gui-auto-video-editor.svg)](https://crates.io/crates/ai-gui-auto-video-editor)

AGAVE (AI GUI Auto Video Editor) is a command-line and GUI tool for automated video editing using AI. Designed for content creators who want to drop in raw footage and get polished results without manual editing.

![AI Video Editor GUI](assets/Screenshot_20260319_124018.png)

> **Screenshot may show an older UI.** Current version uses sharp rectangular edges and a red-accent dark theme.

## Quick Start

**GUI** (default):
```bash
cargo run
```

**CLI** (from terminal with arguments):
```bash
cargo run --release -- -i input.mp4 -o output.mp4 --preset youtube
```

**Watch/daemon mode** (headless, for servers):
```bash
cargo run -- --headless
```

**Using just:**
```bash
just gui      # Run GUI explicitly
just watch    # Run in headless watch mode
just build    # Build release
just test     # Run tests
```

## Installation

### From crates.io

```bash
cargo install ai-gui-auto-video-editor
```

### From Source

```bash
git clone https://github.com/DraconDev/ai-gui-auto-video-editor.git
cd agave
./install.sh --user    # Install to ~/.local/bin (no sudo)
# or
sudo ./install.sh      # Install to /usr/local/bin
```

The install script will:
- Build and install the binary
- Install the application icon and desktop entry (shows in app menu)
- Optionally set up a systemd service for daemon mode

### Releases & Distribution

- Download pre-built binaries from [GitHub Releases](https://github.com/DraconDev/ai-gui-auto-video-editor/releases).
- Run `scripts/release.sh` to build, bundle, and checksum a release tarball locally.
- Also available on [crates.io](https://crates.io/crates/ai-gui-auto-video-editor): `cargo install ai-gui-auto-video-editor`.

### Requirements

- [Rust](https://rustup.rs/) (edition 2024)
- [FFmpeg](https://ffmpeg.org/) (for video processing, includes ffprobe)

### NixOS

```bash
nix-shell  # or: nix develop
```

## Usage

### GUI Mode

Launch without arguments from desktop or run:
```bash
agave
agave --gui    # Explicit
```

**Configure once, walk away:**
```bash
agave --gui --start-minimized
```
Starts watching configured folders with no window visible. Desktop notifications are sent when files complete. Configure folders via GUI when needed.

The GUI provides a visual interface for managing watch folders and settings.

### CLI Mode

```bash
# Batch process a directory
agave -I ./raw_videos -O ./edited --preset youtube

# Parallel batch processing (4 workers)
agave -I ./raw_videos -O ./edited --parallel-workers 4

# Resume an interrupted batch (progress is auto-saved)
agave -I ./raw_videos -O ./edited
```

## CLI Options

| Flag | Description |
|------|-------------|
| `-i, --input-file <FILE>` | Input video file |
| `-I, --input-dir <DIR>` | Input directory (batch mode) |
| `-o, --output-file <FILE>` | Output video file |
| `-O, --output-dir <DIR>` | Output directory (batch mode) |
| `-P, --preset <PRESET>` | Preset: `youtube`, `shorts`, `tiktok`, `reels`, `podcast`, `twitter`, `minimal` |
| `-c, --config <FILE>` | Path to TOML config file |
| `--gui` | Launch graphical interface |
| `--start-minimized` | Start GUI minimized (no window, keeps watching in background) |
| `--clear-progress` | Clear batch progress before processing |
| `--notify` | Send desktop notifications |
| `-w, --watch <DIR>` | Watch folder for new videos |
| `--headless` | Run in watch/daemon mode (no GUI) |
| `--start-minimized` | Start GUI minimized (background watch) |
| `-n, --dry-run` | Preview without processing |
| `-j, --json` | JSON output for scripting |
| `--generate-config` | Output sample config |
| `-v, --verbose` | Increase verbosity (-v, -vv) |
| `-q, --quiet` | Suppress output |
| `--no-progress` | Disable progress bars |

### Processing Options

| Flag | Description |
|------|-------------|
| `-t, --threshold <dB>` | Silence threshold (default: -30.0) |
| `-d, --duration <SEC>` | Min silence duration (default: 0.5) |
| `-p, --padding <SEC>` | Padding around cuts (default: 0.1) |
| `-s, --speedup` | Speed up silences instead of cutting |
| `-E, --enhance` | Enable audio enhancement |
| `--noise-reduction` | Enable noise reduction |
| `-m, --music <FILE>` | Background music file |
| `--music-dir <DIR>` | Music folder (picks random track) |
| `--intro <FILE>` | Video to prepend |
| `--outro <FILE>` | Video to append |
| `--stabilize` | Enable video stabilization |
| `--color-correct` | Enable auto color correction |
| `--reframe` | Auto-reframe to vertical (9:16) |
| `--blur-background` | Apply uniform boxblur to video (not ML segmentation) |
| `--watermark <FILE>` | Add watermark image overlay |
| `--watermark-position <POS>` | Watermark position (top-left, top-right, bottom-left, bottom-right, center) |
| `--watermark-scale <FLOAT>` | Watermark scale factor (default: 1.0) |
| `--preview` | Generate a quick low-resolution preview |
| `--scene-detect` | Use scene-change detection in addition to silence |
| `--scene-threshold <FLOAT>` | Scene detection threshold (default: 0.3) |
| `--multi-format` | Generate multiple resolution outputs simultaneously |
| `--resolution <RES>` | Target resolution (720p, 1080p, 1440p, 4k, vertical-1080p, vertical-720p) |
| `--parallel-workers <N>` | Parallel batch processing workers (default: 1) |
| `--notify` | Send desktop notifications on completion/error |
| `--gpu <TYPE>` | Hardware acceleration: `auto`, `nvenc`, `amf`, `vaapi`, `videotoolbox`, `none` |

### Export Options

| Flag | Description |
|------|-------------|
| `--export-srt` | Generate SRT subtitles (Whisper transcription) |
| `--export-captions` | Burn styled subtitles into video |
| `--export-chapters` | Generate YouTube chapters (from Whisper) |
| `--export-clips` | Extract highlight clips for Shorts/Reels |
| `--export-fcpxml` | Generate FCPXML |
| `--export-thumbnail` | Generate YouTube thumbnail from best frame |



## Presets

| Preset | Description |
|--------|-------------|
| `youtube` | Cut silences, enhance audio (two-pass loudnorm + gentle EQ), export chapters + FCPXML |
| `shorts` | Speedup silences (3x), enhance audio, extract highlight clips |
| `tiktok` | Vertical 9:16, 4x speedup, -12 LUFS, burn captions |
| `reels` | Vertical 9:16, 3.5x speedup, 90s clips max |
| `twitter` | Landscape 16:9, 2:20 max clips |
| `podcast` | Cut silences, enhance audio (-16 LUFS), export SRT + styled captions |
| `minimal` | Just silence detection, no enhancement |

## Configuration

Create `agave.toml` in your project directory or `~/.config/agave/config.toml`:

Config from `~/.config/agave/config.toml` is loaded automatically — no `--config` flag needed.

```toml
[paths]
input_dir = "watch"
output_dir = "output"
music_dir = "music"

# Watch folders (also used by GUI). CLI reads these automatically.
[[paths.watch_folders]]
input = "/home/user/Videos/youtube"
output = "/home/user/Videos/youtube/output"
preset = "youtube"
enabled = true

[silence]
threshold_db = -30.0
min_duration = 0.5
padding = 0.1
mode = "cut"
speedup_factor = 4.0

[filler_words]
enabled = true
words = ["um", "uh", "ah", "er"]
padding = 0.05

[audio]
enhance = true
noise_reduction = false
target_lufs = -14.0
duck_volume = 0.2

[video]
stabilize = false
color_correct = false
reframe = false
blur_background = false
# Target output resolution: hd720p, fhd1080p, qhd1440p, uhd4k, vertical1080p, vertical720p
target_resolution = "fhd1080p"

[export]
subtitles = false       # Generate SRT subtitles (Whisper transcription)
chapters = false        # Generate YouTube chapters (from transcript)
fcpxml = false          # Generate Final Cut Pro XML
edl = false             # Generate Edit Decision List
captions = false        # Burn styled subtitles into video
clips = false           # Extract highlight clips for Shorts/Reels
clip_count = 3          # Number of clips to extract
clip_min_duration = 15  # Minimum clip duration (seconds)
clip_max_duration = 60  # Maximum clip duration (seconds)
thumbnail = false       # Generate thumbnail image
thumbnail_width = 1280
thumbnail_height = 720
multi_format = false    # Generate multiple resolutions
extra_resolutions = ["hd720p", "vertical1080p"]  # Additional outputs
preview = false         # Generate quick low-res preview alongside output

[watch]
enabled = false
interval = 5
```

### Watch Folders

The `[[paths.watch_folders]]` section works for both CLI and GUI. Drop a video in the configured folder and it gets processed automatically:

```bash
# Uses watch_folders from config — just run:
agave

# Or override with a specific folder:
agave --watch ./incoming -O ./processed
```

Progress is shown with timestamps during processing:
```
[14:30:15] [NEW FILE] "/home/user/Videos/youtube/video.mp4"
[14:30:15] [START] Processing video.mp4...
[14:30:16] [2%] video.mp4 - Analyzing silence
[14:30:25] [10%] video.mp4 - Planning edits
[14:30:25] [15%] video.mp4 - Trimming video
[14:30:45] [78%] video.mp4 - Enhancing audio
[14:30:50] [DONE] video.mp4 -> output/video.mp4 (35.2s)
```

## Build Options

```bash
# Build everything (CLI + GUI) - default
cargo build --release

# Build CLI only (smaller binary, no GUI dependencies)
cargo build --release --no-default-features --features cli

# Build with specific features
cargo build --features cli,gui
```

## Testing

- `cargo test --all-features` exercises config parsing, presets, silence detection, ML helpers, exporters, and CLI/GUI glue. `scripts/release.sh` already runs that plus `cargo clippy --all-features` before packaging each release.
- For localized checks run `cargo test config::tests::test_preset_youtube` or `cargo test --package agave -- ml` to focus the suite on configuration/ML helpers.

## Project Status

| Feature | Status |
|---------|--------|
| Silence detection | Done |
| Silence trimming | Done |
| Speedup mode | Done |
| Batch processing | Done |
| **Parallel batch processing** | Done |
| TOML config | Done |
| Audio enhancement | Done |
| Noise reduction | Done |
| Music mixing | Done |
| Intro/Outro | Done |
| Video stabilization | Done |
| Auto color correction | Done |
| **Scene-change detection** | Done |
| Preset profiles | Done |
| **Social media presets** (TikTok, Reels, Twitter) | Done |
| Watch mode | Done |
| Dry run | Done |
| JSON output | Done |
| Export formats | Done |
| **Thumbnail generation** | Done |
| **Watermark overlay** | Done |
| **Preview generation** | Done |
| **Multi-format output** | Done |
| Whisper STT | Done |
| Filler word removal | Done |
| Auto-reframe | Done |
| Background blur | Done |
| GUI (egui) | Done |
| Desktop notifications | Done |
| Desktop entry / app menu | Done |
| Unified binary (CLI + GUI) | Done |

## License

This project is dual-licensed:

- **AGPL-3.0-only** — See [LICENSE](LICENSE) for the full text. This is the default license for open source use.
- **Commercial License** — For organizations that prefer not to comply with AGPLv3's source disclosure requirements. See [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md) for details.

By contributing to this project, you agree to the terms in [CLA.md](CLA.md).