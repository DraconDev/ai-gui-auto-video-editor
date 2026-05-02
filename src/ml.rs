//! ML-based video processing features
//!
//! This module provides:
//! - Face detection for auto-reframe
//! - Person segmentation for background blur
//!
//! Models are lazy-loaded to minimize memory usage when features aren't used.

use anyhow::Result;
use image::GenericImageView;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tracing::info;
use tract_onnx::prelude::*;

/// Frame extraction utilities
pub struct FrameExtractor;

impl FrameExtractor {
    /// Extract frames from video at specified intervals
    /// Returns paths to extracted frame images
    pub fn extract_frames(
        video_path: &Path,
        output_dir: &Path,
        interval_fps: f32,
    ) -> Result<Vec<std::path::PathBuf>> {
        std::fs::create_dir_all(output_dir)?;

        let path_str = video_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Video path contains invalid UTF-8 characters"))?;
        let out_dir_str = output_dir
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Output path contains invalid UTF-8 characters"))?;

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                path_str,
                "-vf",
                &format!("fps={}", interval_fps),
                "-y",
                &format!("{}/frame_%04d.png", out_dir_str),
            ])
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to extract frames from video");
        }

        // Collect extracted frame paths
        let mut frames = vec![];
        for entry in std::fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "png").unwrap_or(false) {
                frames.push(path);
            }
        }

        frames.sort();
        Ok(frames)
    }

    /// Get video dimensions (width, height)
    pub fn get_video_dimensions(video_path: &Path) -> Result<(u32, u32)> {
        let path_str = video_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Video path contains invalid UTF-8 characters"))?;

        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=p=0",
                path_str,
            ])
            .output()?;

        let dims = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = dims.trim().split(',').collect();

        if parts.len() == 2 {
            let width: u32 = parts[0].parse()?;
            let height: u32 = parts[1].parse()?;
            Ok((width, height))
        } else {
            anyhow::bail!("Failed to parse video dimensions");
        }
    }

    /// Get video frames per second (FPS) using ffprobe
    pub fn get_video_fps(video_path: &Path) -> Result<f32> {
        let path_str = video_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Video path contains invalid UTF-8 characters"))?;

        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=r_frame_rate",
                "-of",
                "csv=p=0",
                path_str,
            ])
            .output()?;

        let fps_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = fps_str.trim().split('/').collect();
        let fps = if parts.len() == 2 {
            let num: f32 = parts[0].parse().unwrap_or(25.0);
            let den: f32 = parts[1].parse().unwrap_or(1.0);
            if den > 0.0 { num / den } else { 25.0 }
        } else {
            fps_str.trim().parse::<f32>().unwrap_or(25.0)
        };
        Ok(fps)
    }

    /// Get video duration in seconds
    pub fn get_video_duration(video_path: &Path) -> Result<f32> {
        let path_str = video_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Video path contains invalid UTF-8 characters"))?;

        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                path_str,
            ])
            .output()?;

        let duration = String::from_utf8_lossy(&output.stdout);
        duration
            .trim()
            .parse::<f32>()
            .map_err(|e| anyhow::anyhow!("Failed to parse duration: {}", e))
    }
}

/// Model IDs on HuggingFace Hub
/// Using existing public models instead of custom uploads
const FACE_MODEL_ID: &str = "onnx-models/ultra-light-face-detector";
const FACE_MODEL_FILE: &str = "version-RFB-320.onnx";
const SEGMENT_MODEL_ID: &str = "dhkim2810/MODNet";
const SEGMENT_MODEL_FILE: &str = "modnet.onnx";

/// Type alias for the ONNX model
type OnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Face detector using ONNX model
pub struct FaceDetector {
    model: Arc<OnnxModel>,
}

impl FaceDetector {
    /// Load the face detection model
    ///
    /// Model is downloaded on first use if not present.
    /// Uses MediaPipe or similar lightweight face detection model.
    pub fn load() -> Result<Self> {
        // Downloads model from HuggingFace on first use, then caches locally
        let model_path = Self::get_model_path()?;

        if !model_path.exists() {
            Self::download_model(&model_path)?;
        }

        let model = tract_onnx::onnx()
            .model_for_path(&model_path)?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            model: Arc::new(model),
        })
    }

    /// Get the path where the model is stored
    fn get_model_path() -> Result<std::path::PathBuf> {
        let cache_dir = directories::ProjectDirs::from("com", "ai-vid-editor", "ai-vid-editor")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        Ok(cache_dir.join("face_detection.onnx"))
    }

    /// Download the model if not present
    fn download_model(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Downloading face detection model from HuggingFace...");

        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(FACE_MODEL_ID.to_string());
        let downloaded = repo.get(FACE_MODEL_FILE)?;

        let temp_path = path.with_extension("tmp");
        std::fs::copy(&downloaded, &temp_path)?;
        std::fs::rename(&temp_path, path)?;

        info!(path = ?path, "Model downloaded");
        Ok(())
    }

    /// Detect faces in a frame
    ///
    /// Returns a list of bounding boxes (x, y, width, height) normalized to 0-1
    pub fn detect(&self, frame: &image::DynamicImage) -> Result<Vec<FaceBox>> {
        // Preprocess image for the model
        let input = Self::preprocess(frame)?;

        // Run inference
        let result = self.model.run(tvec!(input.into()))?;

        // Parse output into face boxes
        Self::parse_output(&result)
    }

    /// Preprocess image for the model
    fn preprocess(image: &image::DynamicImage) -> Result<Tensor> {
        // Resize to model input size (typically 320x320 or 640x480)
        let resized = image.resize_exact(320, 320, image::imageops::FilterType::Triangle);

        // Convert to RGB and normalize
        let rgb = resized.to_rgb8();
        let data: Vec<f32> = rgb
            .pixels()
            .flat_map(|p| p.0.iter().map(|&v| v as f32 / 255.0))
            .collect();

        // Create tensor with shape [1, 3, 320, 320]
        let tensor = Tensor::from_shape(&[1, 3, 320, 320], &data)?;

        Ok(tensor)
    }

    /// Parse model output into face boxes
    fn parse_output(output: &[TValue]) -> Result<Vec<FaceBox>> {
        // Ultra-light-face-detector outputs:
        // - scores: [1, num_anchors] or [1, num_anchors, 1]
        // - boxes: [1, num_anchors, 4] in [x1, y1, x2, y2] normalized format

        if output.len() < 2 {
            return Ok(vec![]);
        }

        let scores = output[0].to_array_view::<f32>()?;
        let boxes = output[1].to_array_view::<f32>()?;

        let confidence_threshold = 0.5;
        let mut faces = Vec::new();

        // Determine score tensor shape
        let score_dims = scores.shape();
        let num_faces = if score_dims.len() == 2 || score_dims.len() == 3 {
            score_dims[1]
        } else {
            return Ok(vec![]);
        };

        // Boxes shape: [1, num_faces, 4] or flattened
        let box_dims = boxes.shape();
        let boxes_are_flat = box_dims.len() == 2 && box_dims[1] == num_faces * 4;

        for i in 0..num_faces {
            let score = scores[i];
            if score < confidence_threshold {
                continue;
            }

            let (x1, y1, x2, y2) = if boxes_are_flat || box_dims.len() == 3 {
                (
                    boxes[i * 4],
                    boxes[i * 4 + 1],
                    boxes[i * 4 + 2],
                    boxes[i * 4 + 3],
                )
            } else {
                continue;
            };

            // Convert [x1, y1, x2, y2] to [x, y, width, height] normalized
            let x = x1.clamp(0.0, 1.0);
            let y = y1.clamp(0.0, 1.0);
            let width = (x2 - x1).clamp(0.0, 1.0 - x);
            let height = (y2 - y1).clamp(0.0, 1.0 - y);

            faces.push(FaceBox {
                x,
                y,
                width,
                height,
                confidence: score,
            });
        }

        // Sort by confidence (highest first)
        faces.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(faces)
    }
}

/// Bounding box for a detected face
#[derive(Debug, Clone, Copy)]
pub struct FaceBox {
    /// X coordinate (0-1, normalized)
    pub x: f32,
    /// Y coordinate (0-1, normalized)
    pub y: f32,
    /// Width (0-1, normalized)
    pub width: f32,
    /// Height (0-1, normalized)
    pub height: f32,
    /// Confidence score (0-1)
    pub confidence: f32,
}

impl FaceBox {
    /// Get the center of the face box
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Person segmentation for background blur
pub struct PersonSegmenter {
    model: Arc<OnnxModel>,
}

impl PersonSegmenter {
    /// Load the segmentation model
    pub fn load() -> Result<Self> {
        let model_path = Self::get_model_path()?;

        if !model_path.exists() {
            Self::download_model(&model_path)?;
        }

        let model = tract_onnx::onnx()
            .model_for_path(&model_path)?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            model: Arc::new(model),
        })
    }

    fn get_model_path() -> Result<std::path::PathBuf> {
        let cache_dir = directories::ProjectDirs::from("com", "ai-vid-editor", "ai-vid-editor")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        Ok(cache_dir.join("person_segmentation.onnx"))
    }

    fn download_model(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Downloading person segmentation model from HuggingFace...");

        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(SEGMENT_MODEL_ID.to_string());
        let downloaded = repo.get(SEGMENT_MODEL_FILE)?;

        let temp_path = path.with_extension("tmp");
        std::fs::copy(&downloaded, &temp_path)?;
        std::fs::rename(&temp_path, path)?;

        info!(path = ?path, "Model downloaded");
        Ok(())
    }

    /// Segment person from background
    ///
    /// Returns a mask where 1.0 = person, 0.0 = background
    pub fn segment(&self, frame: &image::DynamicImage) -> Result<SegmentationMask> {
        let input = Self::preprocess(frame)?;
        let result = self.model.run(tvec!(input.into()))?;
        Self::parse_output(&result, frame.width(), frame.height())
    }

    fn preprocess(image: &image::DynamicImage) -> Result<Tensor> {
        let resized = image.resize_exact(512, 512, image::imageops::FilterType::Triangle);
        let rgb = resized.to_rgb8();
        let data: Vec<f32> = rgb
            .pixels()
            .flat_map(|p| p.0.iter().map(|&v| v as f32 / 255.0))
            .collect();

        let tensor = Tensor::from_shape(&[1, 3, 512, 512], &data)?;
        Ok(tensor)
    }

    fn parse_output(output: &[TValue], width: u32, height: u32) -> Result<SegmentationMask> {
        // MODNet outputs a matte/mask tensor: [1, 1, H, W] or [1, H, W]
        // Values are 0.0 (background) to 1.0 (foreground/person)

        if output.is_empty() {
            return Ok(SegmentationMask {
                data: vec![0.0; (width * height) as usize],
                width,
                height,
            });
        }

        let mask_tensor = output[0].to_array_view::<f32>()?;
        let dims = mask_tensor.shape();

        // Determine mask dimensions from tensor
        let (mask_h, mask_w) = match dims.len() {
            4 => (dims[2], dims[3]),
            3 => (dims[1], dims[2]),
            2 => (dims[0], dims[1]),
            _ => {
                return Ok(SegmentationMask {
                    data: vec![0.0; (width * height) as usize],
                    width,
                    height,
                });
            }
        };

        // Resize mask to original frame dimensions
        // Simple bilinear interpolation
        let mut data = vec![0.0; (width * height) as usize];

        let scale_x = mask_w as f32 / width as f32;
        let scale_y = mask_h as f32 / height as f32;

        for y in 0..height {
            for x in 0..width {
                let src_x = (x as f32 * scale_x).min(mask_w as f32 - 1.0) as usize;
                let src_y = (y as f32 * scale_y).min(mask_h as f32 - 1.0) as usize;

                let src_idx = src_y * mask_w + src_x;
                let value = if src_idx < mask_tensor.len() {
                    mask_tensor[src_idx]
                } else {
                    0.0
                };

                data[(y * width + x) as usize] = value.clamp(0.0, 1.0);
            }
        }

        Ok(SegmentationMask {
            data,
            width,
            height,
        })
    }
}

/// Segmentation mask for person/background separation
pub struct SegmentationMask {
    /// Mask data (0.0 = background, 1.0 = person)
    pub data: Vec<f32>,
    /// Width of the mask
    pub width: u32,
    /// Height of the mask
    pub height: u32,
}

impl SegmentationMask {
    /// Get the value at a specific pixel
    pub fn get(&self, x: u32, y: u32) -> f32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else {
            0.0
        }
    }
}

/// Crop region for auto-reframe
#[derive(Debug, Clone, Copy)]
pub struct CropRegion {
    /// X offset (0-1, normalized to video width)
    pub x: f32,
    /// Y offset (0-1, normalized to video height)
    pub y: f32,
    /// Width of crop (0-1)
    pub width: f32,
    /// Height of crop (0-1)
    pub height: f32,
}

impl CropRegion {
    /// Create a center crop for 9:16 aspect ratio from any video
    pub fn center_crop_9_16(video_aspect: f32) -> Self {
        // Target aspect = 9/16 = 0.5625 (vertical video)
        // For a video with aspect = w/h, crop width ratio = target_aspect / video_aspect
        let target_aspect = 9.0 / 16.0;
        let crop_width = if video_aspect > 0.0 {
            (target_aspect / video_aspect).min(1.0)
        } else {
            target_aspect
        };
        Self {
            x: (1.0 - crop_width) / 2.0, // Center horizontally
            y: 0.0,
            width: crop_width,
            height: 1.0,
        }
    }

    /// Create crop region following a face
    pub fn from_face(face: &FaceBox, video_aspect: f32) -> Self {
        let target_aspect = 9.0 / 16.0;

        if !video_aspect.is_finite() || video_aspect <= 0.0 {
            return Self::center_crop_9_16(video_aspect);
        }

        let crop_width = target_aspect / video_aspect;

        let face_center_x = face.x + face.width / 2.0;

        let mut crop_x = face_center_x - crop_width / 2.0;

        crop_x = crop_x.max(0.0).min(1.0 - crop_width);

        Self {
            x: crop_x,
            y: 0.0,
            width: crop_width,
            height: 1.0,
        }
    }
}

/// Auto-reframe processor
pub struct AutoReframeProcessor {
    detector: FaceDetector,
}

impl AutoReframeProcessor {
    /// Create a new auto-reframe processor
    pub fn new() -> Result<Self> {
        let detector = FaceDetector::load()?;
        Ok(Self { detector })
    }

    /// Analyze video and generate crop regions for each frame
    pub fn analyze_video(
        &self,
        video_path: &Path,
        sample_fps: f32,
    ) -> Result<Vec<(f32, CropRegion)>> {
        let temp_dir = crate::utils::TempDir::new("ai-vid-editor-frames")?;
        let frames = FrameExtractor::extract_frames(video_path, temp_dir.path(), sample_fps)?;

        let _video_duration = FrameExtractor::get_video_duration(video_path)?;
        let (video_width, video_height) = FrameExtractor::get_video_dimensions(video_path)?;
        let video_aspect = if video_height > 0 {
            video_width as f32 / video_height as f32
        } else {
            16.0 / 9.0
        };

        let mut crop_regions = Vec::new();

        for (i, frame_path) in frames.iter().enumerate() {
            let timestamp = (i as f32) / sample_fps;

            // Load frame
            let frame = image::open(frame_path)?;

            // Detect faces
            let faces = self.detector.detect(&frame)?;

            // Determine crop region
            let crop = if let Some(main_face) = faces.first() {
                CropRegion::from_face(main_face, video_aspect)
            } else {
                // No face detected, use center crop
                CropRegion::center_crop_9_16(video_aspect)
            };

            crop_regions.push((timestamp, crop));
        }

        Ok(crop_regions)
    }

    /// Generate ffmpeg filter for smooth crop following faces.
    /// Uses temporally smoothed crop values with linear interpolation.
    /// `target_resolution` controls the output scale dimensions.
    pub fn generate_crop_filter(
        &self,
        crop_regions: &[(f32, CropRegion)],
        _video_width: u32,
        _video_height: u32,
        target_resolution: crate::config::VideoResolution,
    ) -> String {
        let (scale_w, scale_h) = target_resolution.dimensions();
        if crop_regions.is_empty() {
            return format!("crop=ih*9/16:ih,scale={}:{}", scale_w, scale_h);
        }

        if crop_regions.len() == 1 {
            let region = &crop_regions[0].1;
            return format!(
                "crop=iw*{}:ih:iw*{}:0,scale={}:{}",
                region.width, region.x, scale_w, scale_h
            );
        }

        let smoothed = Self::smooth_crop_regions(crop_regions, 5);
        let first_time = smoothed[0].0;
        let last_time = smoothed[smoothed.len() - 1].0;
        let duration = last_time - first_time;

        if duration == 0.0 {
            let region = &smoothed[0].1;
            return format!(
                "crop=iw*{}:ih:iw*{}:0,scale={}:{}",
                region.width, region.x, scale_w, scale_h
            );
        }

        let first = &smoothed[0].1;
        let last = &smoothed[smoothed.len() - 1].1;
        let x0 = first.x;
        let x1 = last.x;
        let w0 = first.width;
        let w1 = last.width;

        format!(
            "crop=iw*({w0}+({w1}-{w0})*t/{duration}):ih:iw*({x0}+({x1}-{x0})*t/{duration}):0,scale={}:{}",
            scale_w, scale_h
        )
    }

    fn smooth_crop_regions(regions: &[(f32, CropRegion)], window: usize) -> Vec<(f32, CropRegion)> {
        if regions.len() <= window {
            return regions.to_vec();
        }
        let half = window / 2;
        let mut result = Vec::with_capacity(regions.len());
        for i in 0..regions.len() {
            let start = i.saturating_sub(half);
            let end = (i + half + 1).min(regions.len());
            let slice = &regions[start..end];
            let avg_x: f32 = slice.iter().map(|(_, r)| r.x).sum::<f32>() / slice.len() as f32;
            let avg_width: f32 =
                slice.iter().map(|(_, r)| r.width).sum::<f32>() / slice.len() as f32;
            let (t, first) = regions[i];
            result.push((
                t,
                CropRegion {
                    x: avg_x,
                    y: first.y,
                    width: avg_width,
                    height: first.height,
                },
            ));
        }
        result
    }
}

/// Background blur processor
pub struct BackgroundBlurProcessor {
    segmenter: PersonSegmenter,
}

impl BackgroundBlurProcessor {
    /// Create a new background blur processor
    pub fn new() -> Result<Self> {
        let segmenter = PersonSegmenter::load()?;
        Ok(Self { segmenter })
    }

    /// Process a single frame, returning the blurred version
    pub fn process_frame(
        &self,
        frame: &image::DynamicImage,
        blur_strength: u32,
    ) -> Result<image::DynamicImage> {
        // Get segmentation mask
        let mask = self.segmenter.segment(frame)?;

        // Apply blur to the entire frame
        let blurred = frame.blur(blur_strength as f32);

        // Composite: person from original, background from blurred
        let mut result = frame.to_rgb8();
        let blurred_rgb = blurred.to_rgb8();

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let mask_val = mask.get(x, y);
                let original = frame.get_pixel(x, y);
                let blurred_px = blurred_rgb.get_pixel(x, y);

                // Blend based on mask (1.0 = person, 0.0 = background)
                let r = (original.0[0] as f32 * mask_val
                    + blurred_px.0[0] as f32 * (1.0 - mask_val)) as u8;
                let g = (original.0[1] as f32 * mask_val
                    + blurred_px.0[1] as f32 * (1.0 - mask_val)) as u8;
                let b = (original.0[2] as f32 * mask_val
                    + blurred_px.0[2] as f32 * (1.0 - mask_val)) as u8;

                result.put_pixel(x, y, image::Rgb([r, g, b]));
            }
        }

        Ok(image::DynamicImage::ImageRgb8(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_box_center() {
        let face = FaceBox {
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.4,
            confidence: 0.9,
        };

        let (cx, cy) = face.center();
        assert!((cx - 0.25).abs() < 0.001);
        assert!((cy - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_face_box_center_at_origin() {
        let face = FaceBox {
            x: 0.0,
            y: 0.0,
            width: 0.2,
            height: 0.2,
            confidence: 0.5,
        };
        let (cx, cy) = face.center();
        assert!((cx - 0.1).abs() < 0.001);
        assert!((cy - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_segmentation_mask_get_valid() {
        let mask = SegmentationMask {
            data: vec![0.0, 0.5, 1.0, 0.25],
            width: 2,
            height: 2,
        };
        assert_eq!(mask.get(0, 0), 0.0);
        assert_eq!(mask.get(1, 0), 0.5);
        assert_eq!(mask.get(0, 1), 1.0);
        assert_eq!(mask.get(1, 1), 0.25);
    }

    #[test]
    fn test_segmentation_mask_get_out_of_bounds() {
        let mask = SegmentationMask {
            data: vec![0.5; 100],
            width: 10,
            height: 10,
        };
        assert_eq!(mask.get(99, 0), 0.0); // x too large
        assert_eq!(mask.get(0, 99), 0.0); // y too large
        assert_eq!(mask.get(5, 5), 0.5); // valid
    }

    #[test]
    fn test_segmentation_mask_get_boundary() {
        let mask = SegmentationMask {
            data: vec![0.9; 100],
            width: 10,
            height: 10,
        };
        assert_eq!(mask.get(9, 9), 0.9); // last valid pixel
        assert_eq!(mask.get(10, 9), 0.0); // just past edge
    }

    #[test]
    fn test_crop_region_center_crop_16_9() {
        // 16:9 video: crop is a narrow vertical strip centered
        let region = CropRegion::center_crop_9_16(16.0 / 9.0);
        // crop_width = 0.5625 / 1.78 = 0.316 < 1.0
        assert!(region.x > 0.0 && region.x < 1.0);
        assert_eq!(region.y, 0.0);
        assert!(region.width > 0.0 && region.width < 1.0);
        assert_eq!(region.height, 1.0);
        // Centered horizontally
        assert!((region.x - (1.0 - region.width) / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_crop_region_center_crop_4_3() {
        // 4:3 video (aspect 1.33), target 9:16 (aspect 0.5625)
        // crop_width = 0.5625 / 1.33 = 0.423
        let region = CropRegion::center_crop_9_16(4.0 / 3.0);
        assert!(region.x > 0.0 && region.x < 1.0);
        assert_eq!(region.y, 0.0);
        assert!(region.width > 0.0 && region.width < 1.0);
        assert_eq!(region.height, 1.0);
        // Centered horizontally
        assert!((region.x - (1.0 - region.width) / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_crop_region_center_crop_wide() {
        // Very wide video
        let region = CropRegion::center_crop_9_16(21.0 / 9.0);
        // crop_width = 0.5625 / 2.33 = 0.24, centered
        assert!(region.x > 0.0);
        assert!(region.width < 1.0);
    }

    #[test]
    fn test_crop_region_center_crop_narrow() {
        // Very narrow (tall) video
        let region = CropRegion::center_crop_9_16(9.0 / 21.0);
        // crop_width = 0.5625 / 0.428 = 1.31, clamped to 1.0
        assert_eq!(region.width, 1.0);
        assert_eq!(region.x, 0.0);
    }

    #[test]
    fn test_crop_region_from_face_centered() {
        let face = FaceBox {
            x: 0.4,
            y: 0.3,
            width: 0.2,
            height: 0.3,
            confidence: 0.9,
        };
        let region = CropRegion::from_face(&face, 16.0 / 9.0);
        // Face center X = 0.4 + 0.1 = 0.5
        // crop should be centered on 0.5
        assert!(region.x > 0.0 && region.x < 1.0);
        assert_eq!(region.y, 0.0);
        assert!(region.width > 0.0 && region.width < 1.0);
        assert_eq!(region.height, 1.0);
    }

    #[test]
    fn test_crop_region_from_face_edge_clamping() {
        // Face at left edge
        let face = FaceBox {
            x: 0.0,
            y: 0.0,
            width: 0.3,
            height: 0.5,
            confidence: 0.8,
        };
        let region = CropRegion::from_face(&face, 16.0 / 9.0);
        // crop_x should be clamped to 0.0 since face is at left edge
        assert_eq!(region.x, 0.0);
    }

    #[test]
    fn test_crop_region_from_face_right_edge() {
        // Face at right edge - should clamp properly
        let face = FaceBox {
            x: 0.7,
            y: 0.0,
            width: 0.3,
            height: 0.5,
            confidence: 0.8,
        };
        let region = CropRegion::from_face(&face, 16.0 / 9.0);
        // x + width should be <= 1.0 due to clamping
        assert!(region.x + region.width <= 1.0);
    }

    #[test]
    fn test_crop_region_from_face_zero_aspect() {
        // Zero aspect ratio should fall back to center crop
        let face = FaceBox {
            x: 0.4,
            y: 0.3,
            width: 0.2,
            height: 0.3,
            confidence: 0.9,
        };
        let region = CropRegion::from_face(&face, 0.0);
        // Should use center_crop_9_16 fallback
        assert!(region.width > 0.0);
        assert_eq!(region.height, 1.0);
    }

    #[test]
    fn test_crop_region_from_face_negative_aspect() {
        // Negative aspect ratio should fall back to center crop
        let face = FaceBox {
            x: 0.4,
            y: 0.3,
            width: 0.2,
            height: 0.3,
            confidence: 0.9,
        };
        let region = CropRegion::from_face(&face, -1.5);
        // Should use center_crop_9_16 fallback
        assert!(region.width > 0.0);
        assert_eq!(region.height, 1.0);
    }

    #[test]
    fn test_crop_region_from_face_infinite_aspect() {
        // Infinite aspect ratio should fall back to center crop
        let face = FaceBox {
            x: 0.4,
            y: 0.3,
            width: 0.2,
            height: 0.3,
            confidence: 0.9,
        };
        let region = CropRegion::from_face(&face, f32::INFINITY);
        // Falls back to center_crop_9_16 which produces width=0 for infinite aspect
        assert_eq!(region.width, 0.0);
        assert_eq!(region.height, 1.0);
    }

    #[test]
    fn test_center_crop_9_16_wide_video() {
        // Very wide video (21:9) should have a small crop width
        let region = CropRegion::center_crop_9_16(21.0 / 9.0);
        assert!(region.width < 0.5); // crop width should be less than half
        assert_eq!(region.x, (1.0 - region.width) / 2.0); // centered
        assert_eq!(region.height, 1.0);
    }

    #[test]
    fn test_center_crop_9_16_narrow_video() {
        // Narrow video (4:3) should have crop width close to 1.0
        let region = CropRegion::center_crop_9_16(4.0 / 3.0);
        assert!(region.width <= 1.0);
        assert_eq!(region.height, 1.0);
    }

    #[test]
    fn test_center_crop_9_16_zero_aspect() {
        // Zero aspect should not panic
        let region = CropRegion::center_crop_9_16(0.0);
        assert!(region.width > 0.0);
        assert_eq!(region.height, 1.0);
    }

    #[test]
    fn test_generate_crop_filter_empty_regions() {
        let processor = AutoReframeProcessor;
        let regions: &[(f32, CropRegion)] = &[];
        let filter = processor.generate_crop_filter(regions, 1920, 1080, crate::config::VideoResolution::Vertical1080p);
        assert!(filter.contains("crop="));
        assert!(filter.contains("scale="));
    }

    #[test]
    fn test_generate_crop_filter_single_region() {
        let processor = AutoReframeProcessor;
        let region = CropRegion { x: 0.2, y: 0.0, width: 0.5, height: 1.0 };
        let filter = processor.generate_crop_filter(&[(0.0, region)], 1920, 1080, crate::config::VideoResolution::Vertical1080p);
        assert!(filter.contains("crop=iw*0.5:ih:iw*0.2:0"));
        assert!(filter.contains("scale="));
    }

    #[test]
    fn test_generate_crop_filter_multiple_regions() {
        let processor = AutoReframeProcessor;
        let regions = vec![
            (0.0, CropRegion { x: 0.1, y: 0.0, width: 0.4, height: 1.0 }),
            (1.0, CropRegion { x: 0.3, y: 0.0, width: 0.5, height: 1.0 }),
            (2.0, CropRegion { x: 0.5, y: 0.0, width: 0.6, height: 1.0 }),
        ];
        let filter = processor.generate_crop_filter(&regions, 1920, 1080, crate::config::VideoResolution::Vertical1080p);
        // Should produce a linear interpolation expression
        assert!(filter.contains("t/"));
        assert!(filter.contains("scale="));
    }

    #[test]
    fn test_generate_crop_filter_zero_duration() {
        let processor = AutoReframeProcessor;
        let regions = vec![
            (0.0, CropRegion { x: 0.2, y: 0.0, width: 0.5, height: 1.0 }),
            (0.0, CropRegion { x: 0.2, y: 0.0, width: 0.5, height: 1.0 }),
        ];
        let filter = processor.generate_crop_filter(&regions, 1920, 1080, crate::config::VideoResolution::Vertical1080p);
        // Duration=0 should fall back to static crop
        assert!(filter.contains("crop=iw*0.5:ih:iw*0.2:0"));
    }
}
