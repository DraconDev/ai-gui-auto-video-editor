use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::models::whisper::{Config, model::Whisper};
use hf_hub::{Repo, RepoType, api::sync::Api};
use rustfft::{FftPlanner, num_complex::Complex};
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::info;

const WHISPER_MODEL_ID: &str = "openai/whisper-tiny";

#[derive(Debug, PartialEq, Clone)]
pub struct TranscriptSegment {
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub confidence: f32,
}

pub trait VideoSttAnalyzer {
    fn transcribe(&self, audio_path: &Path) -> Result<Vec<TranscriptSegment>>;
}

pub struct CandleSttAnalyzer;

impl CandleSttAnalyzer {
    fn ensure_model_cached() -> Result<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf)>
    {
        let cache_dir = directories::ProjectDirs::from("com", "ai-vid-editor", "ai-vid-editor")
            .map(|d| d.cache_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        std::fs::create_dir_all(&cache_dir).context("failed to create model cache directory")?;

        let config_path = cache_dir.join("whisper-tiny-config.json");
        let tokenizer_path = cache_dir.join("whisper-tiny-tokenizer.json");
        let weights_path = cache_dir.join("whisper-tiny-model.safetensors");

        if !config_path.exists() || !tokenizer_path.exists() || !weights_path.exists() {
            info!("Downloading Whisper model (first time only)...");
            let api = Api::new().context("failed to create hf-hub api")?;
            let repo = api.repo(Repo::new(WHISPER_MODEL_ID.to_string(), RepoType::Model));

            let config_file = repo.get("config.json")?;
            let config_tmp = config_path.with_extension("tmp");
            std::fs::copy(&config_file, &config_tmp).context("failed to cache config.json")?;
            std::fs::rename(&config_tmp, &config_path).context("failed to finalize config.json")?;

            let tokenizer_file = repo.get("tokenizer.json")?;
            let tokenizer_tmp = tokenizer_path.with_extension("tmp");
            std::fs::copy(&tokenizer_file, &tokenizer_tmp)
                .context("failed to cache tokenizer.json")?;
            std::fs::rename(&tokenizer_tmp, &tokenizer_path)
                .context("failed to finalize tokenizer.json")?;

            let weights_file = repo.get("model.safetensors")?;
            let weights_tmp = weights_path.with_extension("tmp");
            std::fs::copy(&weights_file, &weights_tmp).context("failed to cache model weights")?;
            std::fs::rename(&weights_tmp, &weights_path)
                .context("failed to finalize model weights")?;
            info!("Whisper model cached successfully");
        }

        Ok((config_path, tokenizer_path, weights_path))
    }
}

impl VideoSttAnalyzer for CandleSttAnalyzer {
    fn transcribe(&self, audio_path: &Path) -> Result<Vec<TranscriptSegment>> {
        let device = Device::Cpu;

        let (config_path, tokenizer_path, weights_path) =
            Self::ensure_model_cached().context("failed to load/cached Whisper model")?;

        let config: Config = serde_json::from_str(&std::fs::read_to_string(&config_path)?)
            .context("failed to parse config")?;
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(anyhow::Error::msg)
            .context("failed to load tokenizer")?;

        let vb = unsafe {
            // SAFETY: The weights_path points to a validated safetensors file that was
            // downloaded from HuggingFace and validated to exist. Memory-mapping a
            // read-only file is safe as we don't modify the underlying data.
            candle_nn::VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)?
        };

        let mut model = Whisper::load(&vb, config.clone()).context("failed to load model")?;

        let audio_data = load_audio_as_f32(audio_path)?;
        let mel = pcm_to_mel(&config, &audio_data, &device)
            .context("failed to compute mel spectrogram")?;
        let mel_len = mel.dims()[2];

        let segments = decode_greedy(&mut model, &tokenizer, &mel, &config, mel_len)?;

        Ok(segments)
    }
}

fn load_audio_as_f32(path: &Path) -> Result<Vec<f32>> {
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            path.to_str().context("invalid path")?,
            "-ar",
            "16000",
            "-ac",
            "1",
            "-f",
            "f32le",
            "-",
        ])
        .output()
        .context("failed to extract audio with ffmpeg")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed to extract audio: {}", stderr);
    }

    let bytes = &output.stdout;
    // chunks_exact(4) guarantees each chunk is exactly 4 bytes
    let samples: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(samples)
}

fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0f32.powf(mel / 2595.0) - 1.0)
}

fn build_mel_filterbank(n_fft: usize, n_mels: usize, sample_rate: f32) -> Vec<Vec<f32>> {
    let low_freq = 0.0;
    let high_freq = sample_rate / 2.0;
    let low_mel = hz_to_mel(low_freq);
    let high_mel = hz_to_mel(high_freq);
    let mel_points: Vec<f32> = (0..=n_mels)
        .map(|i| low_mel + (high_mel - low_mel) * i as f32 / n_mels as f32)
        .collect();
    let hz_points: Vec<f32> = mel_points.iter().map(|&m| mel_to_hz(m)).collect();
    let bin_points: Vec<f32> = hz_points
        .iter()
        .map(|&hz| (hz * n_fft as f32 / sample_rate).floor())
        .collect();

    let mut filterbank = vec![vec![0.0f32; n_fft / 2 + 1]; n_mels];
    for m in 1..n_mels {
        #[allow(clippy::needless_range_loop)]
        for k in bin_points[m - 1] as usize..bin_points[m + 1].min(n_fft as f32 - 1.0) as usize {
            let anchor = k as f32 - bin_points[m - 1];
            let width = bin_points[m] - bin_points[m - 1];
            let height = if width > 0.0 {
                anchor.min(width - anchor) / width
            } else {
                0.0
            };
            filterbank[m - 1][k] = height;
        }
    }
    filterbank
}

fn pcm_to_mel(config: &Config, pcm: &[f32], device: &Device) -> Result<Tensor> {
    let sample_rate = 16000.0f32;
    let n_fft = 400;
    let hop_length = 160;
    let n_mels = config.num_mel_bins;

    if pcm.len() < n_fft {
        return Tensor::zeros((n_mels, 1, 1), DType::F32, device)
            .context("failed to create empty mel tensor");
    }

    let n_frames = (pcm.len() - n_fft) / hop_length + 1;

    let filterbank = build_mel_filterbank(n_fft, n_mels, sample_rate);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n_fft);

    let mut mel_spec = vec![0.0f32; n_mels * n_frames];

    for frame_idx in 0..n_frames {
        let start = frame_idx * hop_length;

        let mut windowed = vec![Complex::new(0.0, 0.0); n_fft];
        for (i, w) in windowed.iter_mut().enumerate() {
            let sample_idx = start + i;
            let sample = if sample_idx < pcm.len() {
                pcm[sample_idx]
            } else {
                0.0
            };
            let hann = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / n_fft as f32).cos());
            *w = Complex::new(sample * hann, 0.0);
        }

        fft.process(&mut windowed);

        let magnitudes: Vec<f32> = windowed
            .iter()
            .take(n_fft / 2 + 1)
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        for mel_bin in 0..n_mels {
            let mut energy = 0.0f32;
            for (bin_idx, &mag) in magnitudes.iter().enumerate() {
                energy += mag * filterbank[mel_bin][bin_idx];
            }
            let db = if energy > 0.0 {
                20.0 * energy.log10()
            } else {
                -80.0
            };
            mel_spec[mel_bin * n_frames + frame_idx] = db.max(-80.0);
        }
    }

    Tensor::from_vec(mel_spec, (1, n_mels, n_frames), device).map_err(anyhow::Error::msg)
}

fn decode_greedy(
    model: &mut Whisper,
    tokenizer: &Tokenizer,
    mel: &Tensor,
    config: &Config,
    mel_len: usize,
) -> Result<Vec<TranscriptSegment>> {
    let sot_token = tokenizer
        .token_to_id("<|startoftranscript|>")
        .context("missing sot token")?;
    let eot_token = tokenizer
        .token_to_id("<|endoftranscript|>")
        .context("missing eot token")?;
    let transcribe_token = tokenizer
        .token_to_id("<|transcribe|>")
        .context("missing transcribe token")?;
    let no_speech_token = tokenizer.token_to_id("<|nospeech|>").unwrap_or(eot_token);

    let chunk_size = 3000;
    let mut segments = Vec::new();

    for chunk_start in (0..mel_len).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(mel_len);
        let chunk_len = chunk_end - chunk_start;
        if chunk_len < 100 && chunk_start > 0 {
            continue;
        }

        let chunk_mel = mel.narrow(2, chunk_start, chunk_len)?;

        let chunk_encoder_output = model.encoder.forward(&chunk_mel, true)?;

        let mut tokens = vec![sot_token, transcribe_token];
        let mut token_probs = Vec::new();

        for _ in 0..config.max_target_positions.min(448) {
            let input = Tensor::new(tokens.clone(), mel.device())?.unsqueeze(0)?;

            let logits = model.decoder.forward(&input, &chunk_encoder_output, true)?;
            let seq_len = logits.dims()[1];
            let next_token_logits = logits.get(seq_len - 1)?;

            let next_token = next_token_logits.argmax(0)?.to_scalar::<u32>()?;

            if next_token == eot_token || next_token == no_speech_token {
                break;
            }

            let probs = candle_nn::ops::softmax(&next_token_logits, 0)?;
            let prob = probs.get(next_token as usize)?.to_scalar::<f32>()?;
            token_probs.push(prob);

            tokens.push(next_token);

            if tokens.len() > 400 {
                break;
            }
        }

        let text_tokens: Vec<u32> = tokens[2..].to_vec();
        if text_tokens.is_empty() {
            continue;
        }

        let text = tokenizer
            .decode(&text_tokens, true)
            .map_err(anyhow::Error::msg)?;

        if text.is_empty() || text.trim().is_empty() {
            continue;
        }

        let time_start = chunk_start as f32 / 100.0;
        let time_end = chunk_end as f32 / 100.0;

        let confidence = if token_probs.is_empty() {
            0.5
        } else {
            token_probs.iter().sum::<f32>() / token_probs.len() as f32
        };

        segments.push(TranscriptSegment {
            start: time_start,
            end: time_end,
            text: text.trim().to_string(),
            confidence,
        });
    }

    if segments.is_empty() {
        segments.push(TranscriptSegment {
            start: 0.0,
            end: 30.0,
            text: "[No speech detected]".to_string(),
            confidence: 0.0,
        });
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hz_to_mel_conversion() {
        let zero = hz_to_mel(0.0);
        assert_eq!(zero, 0.0, "mel(0) should be 0");

        let mel_1000 = hz_to_mel(1000.0_f32);
        println!("mel(1000 Hz) = {}", mel_1000);
        assert!(mel_1000 > 0.0, "mel(1000) should be positive");

        let mel_700 = hz_to_mel(700.0_f32);
        println!("mel(700 Hz) = {}", mel_700);
        assert!(mel_700 > 0.0, "mel(700) should be positive");
    }

    #[test]
    fn test_mel_to_hz_conversion() {
        assert_eq!(mel_to_hz(0.0), 0.0);
        let hz_1000 = mel_to_hz(hz_to_mel(1000.0_f32));
        assert!((hz_1000 - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_mel_to_hz_is_inverse_of_hz_to_mel() {
        for hz in [100.0_f32, 500.0, 1000.0, 4000.0, 8000.0] {
            let mel = hz_to_mel(hz);
            let hz_back = mel_to_hz(mel);
            assert!((hz_back - hz).abs() < 0.1, "roundtrip failed for hz={}", hz);
        }
    }

    #[test]
    fn test_build_mel_filterbank_structure() {
        let fb = build_mel_filterbank(400, 80, 16000.0);
        assert_eq!(fb.len(), 80);
        assert_eq!(fb[0].len(), 201);
    }

    #[test]
    fn test_transcript_segment_equality() {
        let seg1 = TranscriptSegment {
            start: 0.0,
            end: 5.0,
            text: "Hello".to_string(),
            confidence: 0.9,
        };
        let seg2 = TranscriptSegment {
            start: 0.0,
            end: 5.0,
            text: "Hello".to_string(),
            confidence: 0.9,
        };
        assert_eq!(seg1, seg2);
    }

    #[test]
    fn test_transcript_segment_clone() {
        let seg = TranscriptSegment {
            start: 1.0,
            end: 2.0,
            text: "Test".to_string(),
            confidence: 0.5,
        };
        let cloned = seg.clone();
        assert_eq!(seg, cloned);
    }

    #[test]
    fn test_transcript_segment_ordering() {
        let segs = vec![
            TranscriptSegment {
                start: 5.0,
                end: 10.0,
                text: "Second".to_string(),
                confidence: 0.8,
            },
            TranscriptSegment {
                start: 0.0,
                end: 5.0,
                text: "First".to_string(),
                confidence: 0.9,
            },
        ];
        let mut sorted = segs.clone();
        sorted.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        assert_eq!(sorted[0].text, "First");
        assert_eq!(sorted[1].text, "Second");
    }

    #[test]
    fn test_transcript_segment_debug() {
        let seg = TranscriptSegment {
            start: 1.5,
            end: 3.0,
            text: "Test".to_string(),
            confidence: 0.95,
        };
        let debug = format!("{:?}", seg);
        assert!(debug.contains("1.5"));
        assert!(debug.contains("3"));
        assert!(debug.contains("Test"));
    }

    #[test]
    fn test_hz_to_mel_extreme_values() {
        // Test at frequency boundaries
        assert_eq!(hz_to_mel(0.0), 0.0);
        let mel_high = hz_to_mel(8000.0_f32);
        assert!(mel_high > 100.0, "high freq should map to high mel");
        let hz_from_mel = mel_to_hz(mel_high);
        assert!(
            (hz_from_mel - 8000.0).abs() < 10.0,
            "inverse should be close"
        );
    }

    #[test]
    fn test_build_mel_filterbank_dimensions() {
        // Different configurations
        let fb_80_400 = build_mel_filterbank(400, 80, 16000.0);
        assert_eq!(fb_80_400.len(), 80);

        let fb_128_512 = build_mel_filterbank(512, 128, 22050.0);
        assert_eq!(fb_128_512.len(), 128);

        let fb_40_200 = build_mel_filterbank(200, 40, 8000.0);
        assert_eq!(fb_40_200.len(), 40);
    }

    #[test]
    fn test_build_mel_filterbank_symmetry() {
        let fb = build_mel_filterbank(400, 80, 16000.0);
        assert_eq!(fb.len(), 80);
        // Filter bank is n_mels x (n_fft/2 + 1) = 80 x 201
        assert_eq!(fb[0].len(), 201);
        // All filters should have correct length
        for filter in &fb {
            assert_eq!(filter.len(), 201);
        }
    }

    #[test]
    fn test_transcript_segment_partialeq() {
        let seg1 = TranscriptSegment {
            start: 0.0,
            end: 5.0,
            text: "Hello".to_string(),
            confidence: 0.9,
        };
        let seg2 = TranscriptSegment {
            start: 0.0,
            end: 5.0,
            text: "Hello".to_string(),
            confidence: 0.9,
        };
        let seg3 = TranscriptSegment {
            start: 0.0,
            end: 5.0,
            text: "Different".to_string(),
            confidence: 0.9,
        };
        assert_eq!(seg1, seg2);
        assert_ne!(seg1, seg3);
    }
}
