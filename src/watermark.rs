use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

/// Common system font paths to try for text watermark
const SYSTEM_FONT_PATHS: &[&str] = &[
    // Linux/Debian/Ubuntu
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    // Arch/Manjaro
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    // Fedora/RHEL
    "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
    // macOS
    "/System/Library/Fonts/Helvetica.ttc",
    "/Library/Fonts/Arial.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    // NixOS
    "/run/current-system/sw/share/X11/fonts/truetype/dejavu/DejaVuSans.ttf",
    // Windows (WSL)
    "/mnt/c/Windows/Fonts/arial.ttf",
];

/// Find the first available system font for text watermark
fn find_system_font() -> Option<String> {
    for path in SYSTEM_FONT_PATHS {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    // Fallback: try to find any .ttf font
    if let Ok(entries) = std::fs::read_dir("/usr/share/fonts") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(font) = find_first_ttf(&path)
            {
                return Some(font);
            }
        }
    }
    None
}

fn find_first_ttf(dir: &std::path::Path) -> Option<String> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                == Some("ttf".to_string())
            {
                return path.to_str().map(|s| s.to_string());
            }
            if path.is_dir()
                && let Some(font) = find_first_ttf(&path)
            {
                return Some(font);
            }
        }
    }
    None
}

/// Position for watermark overlay
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WatermarkPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl WatermarkPosition {
    /// Get the ffmpeg overlay position string
    pub fn to_ffmpeg_coords(self, _watermark_w: u32, _watermark_h: u32) -> String {
        let pad = 10; // Padding from edges
        match self {
            WatermarkPosition::TopLeft => format!("{pad}:{pad}"),
            WatermarkPosition::TopRight => format!("W-w-{pad}:{pad}"),
            WatermarkPosition::BottomLeft => format!("{pad}:H-h-{pad}"),
            WatermarkPosition::BottomRight => format!("W-w-{pad}:H-h-{pad}"),
            WatermarkPosition::Center => "(W-w)/2:(H-h)/2".to_string(),
        }
    }

    /// Parse a position string into WatermarkPosition
    pub fn parse_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "top-left" | "topleft" | "tl" => Some(WatermarkPosition::TopLeft),
            "top-right" | "topright" | "tr" => Some(WatermarkPosition::TopRight),
            "bottom-left" | "bottomleft" | "bl" => Some(WatermarkPosition::BottomLeft),
            "bottom-right" | "bottomright" | "br" => Some(WatermarkPosition::BottomRight),
            "center" | "c" | "middle" => Some(WatermarkPosition::Center),
            _ => None,
        }
    }
}

/// Add a watermark/logo overlay to a video
///
/// # Arguments
/// * `input` - Input video path
/// * `watermark` - Watermark image path (PNG with alpha recommended)
/// * `output` - Output video path
/// * `position` - Position of the watermark
/// * `scale` - Scale factor for watermark (1.0 = original size)
pub fn add_watermark(
    input: &Path,
    watermark: &Path,
    output: &Path,
    position: WatermarkPosition,
    scale: f32,
) -> Result<()> {
    info!(?position, scale, "Adding watermark to video");

    let overlay_pos = position.to_ffmpeg_coords(0, 0);

    // Scale the watermark and overlay it
    let filter = if (scale - 1.0).abs() < 0.001 {
        format!("[1:v]format=rgba[wm];[0:v][wm]overlay={}", overlay_pos)
    } else {
        format!(
            "[1:v]scale=iw*{scale_val}:ih*{scale_val},format=rgba[wm];[0:v][wm]overlay={overlay}",
            scale_val = scale,
            overlay = overlay_pos
        )
    };

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().context("invalid input path")?,
            "-i",
            watermark.to_str().context("invalid watermark path")?,
            "-filter_complex",
            &filter,
            "-c:a",
            "copy",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg for watermark")?;

    if !status.success() {
        anyhow::bail!("ffmpeg watermark failed with status: {}", status);
    }

    info!("Watermark added successfully");
    Ok(())
}

/// Add a text watermark to a video
///
/// # Arguments
/// * `input` - Input video path
/// * `output` - Output video path
/// * `text` - Text to overlay
/// * `position` - Position of the text
/// * `font_size` - Font size in pixels
/// * `color` - Text color (e.g., "white", "#FFFFFF")
/// * `opacity` - Opacity from 0.0 to 1.0
pub fn add_text_watermark(
    input: &Path,
    output: &Path,
    text: &str,
    position: WatermarkPosition,
    font_size: u32,
    color: &str,
    opacity: f32,
) -> Result<()> {
    info!(text, ?position, "Adding text watermark to video");

    let escaped_text = text
        .replace('\'', "'\\''")
        .replace(':', "\\:")
        .replace('\\', "\\\\");

    let overlay_pos = match position {
        WatermarkPosition::TopLeft => "x=10:y=10",
        WatermarkPosition::TopRight => "x=w-text_w-10:y=10",
        WatermarkPosition::BottomLeft => "x=10:y=h-text_h-10",
        WatermarkPosition::BottomRight => "x=w-text_w-10:y=h-text_h-10",
        WatermarkPosition::Center => "x=(w-text_w)/2:y=(h-text_h)/2",
    };

    let font_path = find_system_font()
        .unwrap_or_else(|| "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".to_string());

    let escaped_font_path = crate::utils::escape_ffmpeg_filter_path(Path::new(&font_path));

    let filter = format!(
        "drawtext=text='{}':{}:fontsize={}:fontcolor={}@{}:fontfile='{}'",
        escaped_text, overlay_pos, font_size, color, opacity, escaped_font_path
    );

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().context("invalid input path")?,
            "-vf",
            &filter,
            "-c:a",
            "copy",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg for text watermark")?;

    if !status.success() {
        anyhow::bail!("ffmpeg text watermark failed with status: {}", status);
    }

    info!("Text watermark added successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn create_test_video(path: &Path, duration_secs: f32) -> Result<(), String> {
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                &format!("testsrc=duration={}:size=320x240:rate=30", duration_secs),
                "-f",
                "lavfi",
                "-i",
                &format!("sine=frequency=1000:duration={}", duration_secs),
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-crf",
                "28",
                "-c:a",
                "aac",
                "-b:a",
                "32k",
                "-shortest",
                "-y",
                path.to_str().unwrap(),
            ])
            .status()
            .map_err(|_| "ffmpeg not found".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("ffmpeg test video creation failed".to_string())
        }
    }

    fn create_test_image(path: &Path) -> Result<(), String> {
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "color=c=red:size=50x50",
                "-frames:v",
                "1",
                "-y",
                path.to_str().unwrap(),
            ])
            .status()
            .map_err(|_| "ffmpeg not found".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("ffmpeg test image creation failed".to_string())
        }
    }

    #[test]
    fn test_watermark_position_coords() {
        assert_eq!(WatermarkPosition::TopLeft.to_ffmpeg_coords(50, 50), "10:10");
        assert_eq!(
            WatermarkPosition::Center.to_ffmpeg_coords(50, 50),
            "(W-w)/2:(H-h)/2"
        );
    }

    #[test]
    fn test_add_watermark() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let watermark = temp_dir.path().join("watermark.png");
        let output = temp_dir.path().join("output.mp4");

        create_test_video(&video, 2.0).expect("ffmpeg not found");
        create_test_image(&watermark).expect("ffmpeg not found");

        add_watermark(
            &video,
            &watermark,
            &output,
            WatermarkPosition::BottomRight,
            1.0,
        )
        .unwrap();
        assert!(output.exists(), "watermarked output should exist");
    }

    #[test]
    fn test_add_text_watermark() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");

        create_test_video(&video, 2.0).expect("ffmpeg not found");

        add_text_watermark(
            &video,
            &output,
            "Test Watermark",
            WatermarkPosition::BottomRight,
            24,
            "white",
            0.8,
        )
        .unwrap();
        assert!(output.exists(), "text watermarked output should exist");
    }

    #[test]
    fn test_watermark_position_parse_name() {
        assert_eq!(
            WatermarkPosition::parse_name("bottom-right"),
            Some(WatermarkPosition::BottomRight)
        );
        assert_eq!(
            WatermarkPosition::parse_name("top-left"),
            Some(WatermarkPosition::TopLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("center"),
            Some(WatermarkPosition::Center)
        );
        assert_eq!(
            WatermarkPosition::parse_name("tl"),
            Some(WatermarkPosition::TopLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("BR"),
            Some(WatermarkPosition::BottomRight)
        );
        assert_eq!(WatermarkPosition::parse_name("invalid"), None);
    }
}
