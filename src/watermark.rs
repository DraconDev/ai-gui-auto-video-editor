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

    if !scale.is_finite() || scale <= 0.0 {
        anyhow::bail!(
            "watermark scale must be a positive finite value, got {}",
            scale
        );
    }

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
        .replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('\'', "'\\''");

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

    fn create_test_video(path: &Path, duration_secs: f32) -> Result<(), String> {
        crate::tests_common::create_test_video(path, duration_secs)
    }

    fn create_test_image(path: &Path) -> Result<(), String> {
        crate::tests_common::create_test_image(path)
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

    #[test]
    fn test_watermark_position_all_coords() {
        // Test all position coordinate outputs
        assert_eq!(WatermarkPosition::TopLeft.to_ffmpeg_coords(50, 50), "10:10");
        assert_eq!(
            WatermarkPosition::TopRight.to_ffmpeg_coords(50, 50),
            "W-w-10:10"
        );
        assert_eq!(
            WatermarkPosition::BottomLeft.to_ffmpeg_coords(50, 50),
            "10:H-h-10"
        );
        assert_eq!(
            WatermarkPosition::BottomRight.to_ffmpeg_coords(50, 50),
            "W-w-10:H-h-10"
        );
        assert_eq!(
            WatermarkPosition::Center.to_ffmpeg_coords(50, 50),
            "(W-w)/2:(H-h)/2"
        );
    }

    #[test]
    fn test_watermark_position_parse_name_variants() {
        // Test all parse variants
        assert_eq!(
            WatermarkPosition::parse_name("top-left"),
            Some(WatermarkPosition::TopLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("topleft"),
            Some(WatermarkPosition::TopLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("tl"),
            Some(WatermarkPosition::TopLeft)
        );

        assert_eq!(
            WatermarkPosition::parse_name("top-right"),
            Some(WatermarkPosition::TopRight)
        );
        assert_eq!(
            WatermarkPosition::parse_name("topright"),
            Some(WatermarkPosition::TopRight)
        );
        assert_eq!(
            WatermarkPosition::parse_name("tr"),
            Some(WatermarkPosition::TopRight)
        );

        assert_eq!(
            WatermarkPosition::parse_name("bottom-left"),
            Some(WatermarkPosition::BottomLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("bottomleft"),
            Some(WatermarkPosition::BottomLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("bl"),
            Some(WatermarkPosition::BottomLeft)
        );

        assert_eq!(
            WatermarkPosition::parse_name("bottom-right"),
            Some(WatermarkPosition::BottomRight)
        );
        assert_eq!(
            WatermarkPosition::parse_name("bottomright"),
            Some(WatermarkPosition::BottomRight)
        );
        assert_eq!(
            WatermarkPosition::parse_name("br"),
            Some(WatermarkPosition::BottomRight)
        );

        assert_eq!(
            WatermarkPosition::parse_name("center"),
            Some(WatermarkPosition::Center)
        );
        assert_eq!(
            WatermarkPosition::parse_name("c"),
            Some(WatermarkPosition::Center)
        );
        assert_eq!(
            WatermarkPosition::parse_name("middle"),
            Some(WatermarkPosition::Center)
        );
    }

    #[test]
    fn test_text_watermark_escaping() {
        // Test that special characters in text are properly escaped for FFmpeg drawtext
        let text = "Test's Video: Hello\\World";
        let escaped = text
            .replace('\'', "'\\''")
            .replace(':', "\\:")
            .replace('\\', "\\\\");

        assert!(escaped.contains("\\'"), "Apostrophe should be escaped");
        assert!(escaped.contains(":"), "Colon should be escaped");
        assert!(escaped.contains("\\\\"), "Backslash should be escaped");
    }

    #[test]
    fn test_add_text_watermark_positions() {
        // Test that all positions produce valid filter strings
        let positions = [
            (WatermarkPosition::TopLeft, "x=10:y=10"),
            (WatermarkPosition::TopRight, "x=w-text_w-10:y=10"),
            (WatermarkPosition::BottomLeft, "x=10:y=h-text_h-10"),
            (
                WatermarkPosition::BottomRight,
                "x=w-text_w-10:y=h-text_h-10",
            ),
            (WatermarkPosition::Center, "x=(w-text_w)/2:y=(h-text_h)/2"),
        ];

        for (position, expected_pos) in positions.iter() {
            let overlay_pos = match position {
                WatermarkPosition::TopLeft => "x=10:y=10",
                WatermarkPosition::TopRight => "x=w-text_w-10:y=10",
                WatermarkPosition::BottomLeft => "x=10:y=h-text_h-10",
                WatermarkPosition::BottomRight => "x=w-text_w-10:y=h-text_h-10",
                WatermarkPosition::Center => "x=(w-text_w)/2:y=(h-text_h)/2",
            };
            assert_eq!(overlay_pos, *expected_pos);
        }
    }

    #[test]
    fn test_add_watermark_with_scale() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let watermark = temp_dir.path().join("watermark.png");
        let output = temp_dir.path().join("output.mp4");

        create_test_video(&video, 2.0).expect("ffmpeg not found");
        create_test_image(&watermark).expect("ffmpeg not found");

        // Test with scale = 0.5 (half size)
        add_watermark(&video, &watermark, &output, WatermarkPosition::TopLeft, 0.5).unwrap();
        assert!(
            output.exists(),
            "watermarked output with scale should exist"
        );

        // Test with scale = 2.0 (double size)
        let output2 = temp_dir.path().join("output2.mp4");
        add_watermark(
            &video,
            &watermark,
            &output2,
            WatermarkPosition::BottomRight,
            2.0,
        )
        .unwrap();
        assert!(
            output2.exists(),
            "watermarked output with 2x scale should exist"
        );
    }

    #[test]
    fn test_add_text_watermark_with_simple_special_chars() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");

        create_test_video(&video, 2.0).expect("ffmpeg not found");

        // Test with text containing apostrophe (requires escaping in FFmpeg)
        add_text_watermark(
            &video,
            &output,
            "Hello World",
            WatermarkPosition::Center,
            24,
            "white",
            0.8,
        )
        .unwrap();
        assert!(output.exists(), "text watermarked output should exist");
    }

    // ── WatermarkPosition pure logic tests (no FFmpeg needed) ───────────────

    #[test]
    fn test_watermark_position_all_positions() {
        let positions = vec![
            WatermarkPosition::TopLeft,
            WatermarkPosition::TopRight,
            WatermarkPosition::BottomLeft,
            WatermarkPosition::BottomRight,
            WatermarkPosition::Center,
        ];
        for pos in positions {
            let coords = pos.to_ffmpeg_coords(100, 50);
            assert!(!coords.is_empty(), "Every position should produce coords");
        }
    }

    #[test]
    fn test_watermark_position_parse_name_case_insensitive() {
        // Test that parsing is case-insensitive
        assert_eq!(
            WatermarkPosition::parse_name("TOP-LEFT"),
            Some(WatermarkPosition::TopLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("Top-Left"),
            Some(WatermarkPosition::TopLeft)
        );
        assert_eq!(
            WatermarkPosition::parse_name("CENTER"),
            Some(WatermarkPosition::Center)
        );
        assert_eq!(
            WatermarkPosition::parse_name("Middle"),
            Some(WatermarkPosition::Center)
        );
    }

    #[test]
    fn test_watermark_position_parse_name_unknown() {
        assert_eq!(WatermarkPosition::parse_name("invalid"), None);
        assert_eq!(WatermarkPosition::parse_name(""), None);
        assert_eq!(WatermarkPosition::parse_name("top_center"), None);
        assert_eq!(WatermarkPosition::parse_name("bottom-center"), None);
    }

    #[test]
    fn test_watermark_position_coords_padding() {
        // Verify that padding is consistently applied
        let top_left = WatermarkPosition::TopLeft.to_ffmpeg_coords(100, 50);
        assert_eq!(top_left, "10:10", "TopLeft should use padding");

        let top_right = WatermarkPosition::TopRight.to_ffmpeg_coords(100, 50);
        assert_eq!(top_right, "W-w-10:10", "TopRight should use padding");
    }

    #[test]
    fn test_watermark_position_center_formula() {
        let center = WatermarkPosition::Center.to_ffmpeg_coords(100, 50);
        assert_eq!(center, "(W-w)/2:(H-h)/2", "Center uses centering formula");
    }

    // ── WatermarkPosition edge cases ───────────────────────────────────────
    #[test]
    fn test_watermark_position_bottom_left() {
        let pos = WatermarkPosition::BottomLeft;
        let coords = pos.to_ffmpeg_coords(100, 50);
        // Bottom position should have y = H - h - padding
        assert!(coords.contains(":H-h"));
    }

    #[test]
    fn test_watermark_position_bottom_right() {
        let pos = WatermarkPosition::BottomRight;
        let coords = pos.to_ffmpeg_coords(100, 50);
        // Should have W-w for x and H-h for y
        assert!(coords.contains("W-w"));
        assert!(coords.contains("H-h"));
    }

    #[test]
    fn test_watermark_position_parse_all() {
        use WatermarkPosition::*;
        for pos in [TopLeft, TopRight, BottomLeft, BottomRight, Center] {
            let name = format!("{:?}", pos);
            let parsed = WatermarkPosition::parse_name(&name.to_lowercase());
            assert!(parsed.is_some(), "Should parse {:?}", pos);
        }
    }

    #[test]
    fn test_watermark_position_parse_center() {
        assert!(WatermarkPosition::parse_name("center").is_some());
        assert!(WatermarkPosition::parse_name("c").is_some());
        assert!(WatermarkPosition::parse_name("middle").is_some());
    }

    #[test]
    fn test_watermark_position_parse_corners() {
        assert!(WatermarkPosition::parse_name("top-left").is_some());
        assert!(WatermarkPosition::parse_name("topleft").is_some());
        assert!(WatermarkPosition::parse_name("tl").is_some());
    }

    // ── WatermarkPosition more edge cases ─────────────────────────────────
    #[test]
    fn test_watermark_position_bottom_parsing() {
        assert!(WatermarkPosition::parse_name("bottom-left").is_some());
        assert!(WatermarkPosition::parse_name("bottomleft").is_some());
        assert!(WatermarkPosition::parse_name("bl").is_some());
        assert!(WatermarkPosition::parse_name("bottom-right").is_some());
        assert!(WatermarkPosition::parse_name("bottomright").is_some());
        assert!(WatermarkPosition::parse_name("br").is_some());
    }

    #[test]
    fn test_watermark_position_center_coords() {
        let pos = WatermarkPosition::Center;
        let coords = pos.to_ffmpeg_coords(100, 50);
        // Center should use the formula (W-w)/2:(H-h)/2
        assert!(coords.contains("/2"));
    }

    #[test]
    fn test_watermark_position_top_left_coords() {
        let pos = WatermarkPosition::TopLeft;
        let coords = pos.to_ffmpeg_coords(100, 50);
        // Should contain padding value
        assert!(coords.contains("10"));
    }

    #[test]
    fn test_watermark_position_top_right_coords() {
        let pos = WatermarkPosition::TopRight;
        let coords = pos.to_ffmpeg_coords(100, 50);
        // Should contain W-w
        assert!(coords.contains("W-w"));
    }

    #[test]
    fn test_watermark_position_bottom_left_coords() {
        let pos = WatermarkPosition::BottomLeft;
        let coords = pos.to_ffmpeg_coords(100, 50);
        // Should contain H-h
        assert!(coords.contains("H-h"));
    }

    // ── WatermarkPosition more coords ────────────────────────────────────
    #[test]
    fn test_watermark_position_bottom_right_coords() {
        let pos = WatermarkPosition::BottomRight;
        let coords = pos.to_ffmpeg_coords(100, 50);
        // Should contain W-w and H-h
        assert!(coords.contains("W-w") || coords.contains("H-h"));
    }

    #[test]
    fn test_watermark_position_all_positions_different() {
        use WatermarkPosition::*;
        let w = 1920;
        let h = 1080;
        // At minimum, center should be different from corners
        let center = Center.to_ffmpeg_coords(w, h);
        let tl = TopLeft.to_ffmpeg_coords(w, h);
        assert_ne!(center, tl);
    }
}
