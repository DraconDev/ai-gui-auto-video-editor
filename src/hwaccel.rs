use serde::{Deserialize, Serialize};
use std::process::Command;

/// Hardware acceleration backend for video encoding.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HwAccel {
    /// Software encoding (CPU, default)
    #[default]
    None,
    /// NVIDIA NVENC
    Nvenc,
    /// AMD AMF
    Amf,
    /// Intel/AMD VAAPI (Linux)
    Vaapi,
    /// Apple VideoToolbox (macOS)
    VideoToolbox,
}

impl HwAccel {
    /// String representation for CLI/config.
    pub fn as_str(&self) -> &'static str {
        match self {
            HwAccel::None => "none",
            HwAccel::Nvenc => "nvenc",
            HwAccel::Amf => "amf",
            HwAccel::Vaapi => "vaapi",
            HwAccel::VideoToolbox => "videotoolbox",
        }
    }

    /// Human-readable label for UI dropdowns.
    pub fn display_name(&self) -> &'static str {
        match self {
            HwAccel::None => "None (CPU)",
            HwAccel::Nvenc => "NVIDIA NVENC",
            HwAccel::Amf => "AMD AMF",
            HwAccel::Vaapi => "VAAPI (Linux)",
            HwAccel::VideoToolbox => "VideoToolbox (macOS)",
        }
    }

    /// Parse from string (case-insensitive).
    pub fn parse_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(HwAccel::None),
            "nvenc" | "nvidia" => Some(HwAccel::Nvenc),
            "amf" | "amd" => Some(HwAccel::Amf),
            "vaapi" | "intel" => Some(HwAccel::Vaapi),
            "videotoolbox" | "apple" | "mac" => Some(HwAccel::VideoToolbox),
            _ => None,
        }
    }

    /// H.264 codec name for ffmpeg.
    pub fn video_codec(&self) -> &'static str {
        match self {
            HwAccel::None => "libx264",
            HwAccel::Nvenc => "h264_nvenc",
            HwAccel::Amf => "h264_amf",
            HwAccel::Vaapi => "h264_vaapi",
            HwAccel::VideoToolbox => "h264_videotoolbox",
        }
    }

    /// Extra ffmpeg args needed before `-i` for some hwaccels.
    pub fn input_args(&self) -> Vec<&'static str> {
        match self {
            HwAccel::Vaapi => vec![
                "-hwaccel",
                "vaapi",
                "-hwaccel_device",
                "/dev/dri/renderD128",
            ],
            HwAccel::VideoToolbox => vec!["-hwaccel", "videotoolbox"],
            _ => vec![],
        }
    }

    /// Returns true if this accel requires a hwaccel flag on the input side.
    pub fn needs_hwaccel_input(&self) -> bool {
        matches!(self, HwAccel::Vaapi | HwAccel::VideoToolbox)
    }

    /// Auto-detect best available GPU encoder by probing ffmpeg.
    pub fn detect() -> Self {
        let output = match Command::new("ffmpeg").args(["-hwaccels"]).output() {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_lowercase(),
            Err(_) => return HwAccel::None,
        };

        if (output.contains("cuda") || output.contains("nvenc")) && codec_available("h264_nvenc") {
            return HwAccel::Nvenc;
        }
        if output.contains("amf") && codec_available("h264_amf") {
            return HwAccel::Amf;
        }
        if output.contains("vaapi") && codec_available("h264_vaapi") {
            return HwAccel::Vaapi;
        }
        if output.contains("videotoolbox") && codec_available("h264_videotoolbox") {
            return HwAccel::VideoToolbox;
        }

        HwAccel::None
    }
}

/// Check if a codec is available in ffmpeg.
fn codec_available(codec: &str) -> bool {
    match Command::new("ffmpeg").args(["-encoders"]).output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains(codec),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hwaccel_parse_name() {
        assert_eq!(HwAccel::parse_name("none"), Some(HwAccel::None));
        assert_eq!(HwAccel::parse_name("NVENC"), Some(HwAccel::Nvenc));
        assert_eq!(HwAccel::parse_name("nvidia"), Some(HwAccel::Nvenc));
        assert_eq!(HwAccel::parse_name("amf"), Some(HwAccel::Amf));
        assert_eq!(HwAccel::parse_name("amd"), Some(HwAccel::Amf));
        assert_eq!(HwAccel::parse_name("vaapi"), Some(HwAccel::Vaapi));
        assert_eq!(
            HwAccel::parse_name("videotoolbox"),
            Some(HwAccel::VideoToolbox)
        );
        assert_eq!(HwAccel::parse_name("mac"), Some(HwAccel::VideoToolbox));
        assert_eq!(HwAccel::parse_name("unknown"), None);
    }

    #[test]
    fn test_video_codec_mapping() {
        assert_eq!(HwAccel::None.video_codec(), "libx264");
        assert_eq!(HwAccel::Nvenc.video_codec(), "h264_nvenc");
        assert_eq!(HwAccel::Amf.video_codec(), "h264_amf");
        assert_eq!(HwAccel::Vaapi.video_codec(), "h264_vaapi");
        assert_eq!(HwAccel::VideoToolbox.video_codec(), "h264_videotoolbox");
    }

    #[test]
    fn test_default_is_none() {
        let accel: HwAccel = Default::default();
        assert_eq!(accel, HwAccel::None);
    }
}
