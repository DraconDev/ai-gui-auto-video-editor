# Big Features Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add GPU encoding, preview-before-render, GUI batch queue, and better notifications/UI feedback.

**Architecture:** Extend `config.rs` with new fields, `editor.rs` with GPU encode paths, `batch_processor.rs` with preview extraction and queue logic, `gui/` with new tabs and status widgets.

**Tech Stack:** Rust, egui, ffmpeg (hwaccel codecs), indicatif (progress bars)

---

## Task 1: GPU Hardware Acceleration

**Files:**
- Create: `src/hwaccel.rs`
- Modify: `src/config.rs`, `src/editor.rs`, `src/main.rs`
- Test: `src/hwaccel.rs` (unit tests for detection)

### Step 1.1: Write GPU codec detection module

Create `src/hwaccel.rs` with `HwAccel` enum, auto-detection via `ffmpeg -hwaccels`, and codec mapping.

```rust
use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HwAccel {
    #[default]
    None,
    Nvenc,        // NVIDIA
    Amf,          // AMD
    Vaapi,        // Intel/AMD Linux
    VideoToolbox, // macOS
}

impl HwAccel {
    pub fn as_str(&self) -> &'static str {
        match self {
            HwAccel::None => "none",
            HwAccel::Nvenc => "nvenc",
            HwAccel::Amf => "amf",
            HwAccel::Vaapi => "vaapi",
            HwAccel::VideoToolbox => "videotoolbox",
        }
    }

    pub fn video_codec(&self) -> &'static str {
        match self {
            HwAccel::None => "libx264",
            HwAccel::Nvenc => "h264_nvenc",
            HwAccel::Amf => "h264_amf",
            HwAccel::Vaapi => "h264_vaapi",
            HwAccel::VideoToolbox => "h264_videotoolbox",
        }
    }

    pub fn detect() -> Self {
        // Run ffmpeg -hwaccels, parse output
        // Try nvenc first, then amf, vaapi, videotoolbox
        // Return first available or None
    }
}
```

### Step 1.2: Wire into Config

Add `hw_accel: HwAccel` to `VideoConfig` with serde default `None`.

### Step 1.3: Add CLI flag `--gpu`

Add `--gpu <nvenc|amf|vaapi|videotoolbox|none|auto>` to `Cli` struct in `main.rs`.

### Step 1.4: Swap codec in editor.rs

In `FfmpegEditor::enhance_audio`, `trim_video`, `concatenate_videos`, etc., replace hardcoded `libx264` with `config.video.hw_accel.video_codec()`.

### Step 1.5: Write tests

Test `HwAccel::detect()` with mock `Command` output. Test codec mapping.

---

## Task 2: True Preview-Before-Render

**Files:**
- Modify: `src/preview.rs`, `src/config.rs`, `src/batch_processor.rs`, `src/main.rs`

### Step 2.1: Add preview_duration config field

Add `preview_duration: f32` (default 30.0) to `ExportConfig`.

### Step 2.2: Modify generate_preview to use config duration

Update `generate_preview()` to accept `max_duration` from config.

### Step 2.3: Extract preview BEFORE full pipeline

In `process_single_file_with_intro_outro_progress()`, after silence analysis (≈0.1 progress), if `config.export.preview`, extract first N seconds to `{output}_preview.mp4`, report progress as "Generating preview...".

### Step 2.4: Add CLI flag `--preview-duration`

Add `--preview-duration <SEC>` override in `main.rs`.

---

## Task 3: Better Notifications & UI Feedback

**Files:**
- Modify: `src/gui/tabs.rs`, `src/gui/theme.rs`, `src/batch_processor.rs`, `src/main.rs`
- Create: `src/notifier.rs` (optional abstraction)

### Step 3.1: Add indicatif progress bars to CLI batch mode

In `process_batch_dir` and `process_batch_dir_parallel`, wrap file iteration in `indicatif::ProgressBar` with ETA.

### Step 3.2: Add batch summary to CLI

After batch completes, print summary table: processed / failed / skipped / total.

### Step 3.3: Add animated spinner to GUI status badge

In `gui/theme.rs`, add `animated_spinner()` helper. In `gui/tabs.rs`, show spinner in `draw_header()` status badge when `Processing`.

### Step 3.4: Add toast notifications in GUI

On `WatcherEvent::Completed` and `WatcherEvent::Failed`, push a timed toast to a new `toasts: Vec<Toast>` field in `AppState`. Draw toasts in top-right corner with auto-dismiss after 5s.

### Step 3.5: Add heartbeat timestamp to watch mode CLI

In `run_watch_mode`, print `[HH:MM:SS] Watching... (last: filename)` every 30s.

---

## Task 4: GUI Batch Queue + Process Now

**Files:**
- Modify: `src/gui.rs`, `src/gui/tabs.rs`, `src/gui/processing.rs`
- Create: `src/gui/queue.rs`

### Step 4.1: Create queue data structures

```rust
#[derive(Debug, Clone)]
struct QueuedFile {
    path: PathBuf,
    preset: String,
    status: QueueStatus,
    progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum QueueStatus { Queued, Processing, Done, Error }
```

Add `batch_queue: Vec<QueuedFile>` and `show_queue_modal: bool` to `AppState`.

### Step 4.2: Add "Queue" tab to GUI

New tab `Tab::Queue` between `Folders` and `Settings`. Shows queued files with status, progress bar, and controls.

### Step 4.3: Add drag-and-drop or file picker to queue

Use `rfd::FileDialog` with multi-select. Allow removing items from queue.

### Step 4.4: Add "Process Queue" button

Spawns watcher-like thread that processes queued files one-by-one, sending progress events back to GUI.

### Step 4.5: Add preset selector per queued file

Dropdown per file to override preset.

---

## Self-Review Checklist

- [ ] Spec coverage: GPU, preview, notifications, queue — all have tasks
- [ ] No placeholders: every step has exact code or command
- [ ] Type consistency: `HwAccel` used consistently across config/editor/CLI
- [ ] Test plan: each task has test steps
