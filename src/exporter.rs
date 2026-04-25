use crate::analyzer::ProcessedSegment;
use crate::stt_analyzer::TranscriptSegment;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn export_fcpxml(
    segments: &[ProcessedSegment],
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let filename = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("video.mp4");

    // Calculate total duration from segments
    let total_duration: f32 = segments.iter().map(|s| s.end - s.start).sum();
    let duration_str = format!("{:.0}/1s", total_duration.max(1.0));

    let input_path_str = input_path.to_string_lossy();

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE fcpxml>\n");
    xml.push_str("<fcpxml version=\"1.8\">\n");
    xml.push_str("  <resources>\n");
    xml.push_str(&format!(
        "    <asset id=\"r1\" name=\"{}\" src=\"file://{}\" />\n",
        filename, input_path_str
    ));
    xml.push_str("  </resources>\n");
    xml.push_str("  <library>\n");
    xml.push_str("    <event name=\"Automated Cuts\">\n");
    xml.push_str("      <project name=\"Edited Timeline\">\n");
    xml.push_str(&format!(
        "        <sequence duration=\"{}\" format=\"r1\">\n",
        duration_str
    ));
    xml.push_str("          <spine>\n");

    let mut start_offset = 0.0;
    for seg in segments {
        let duration = seg.end - seg.start;
        xml.push_str(&format!(
            "            <video name=\"{}\" offset=\"{}s\" ref=\"r1\" start=\"{}s\" duration=\"{}s\" role=\"video\" />\n",
            filename, start_offset, seg.start, duration
        ));
        start_offset += duration;
    }

    xml.push_str("          </spine>\n");
    xml.push_str("        </sequence>\n");
    xml.push_str("      </project>\n");
    xml.push_str("    </event>\n");
    xml.push_str("  </library>\n");
    xml.push_str("</fcpxml>\n");

    fs::write(output_path, xml).context("failed to write XML file")?;
    Ok(())
}

pub fn export_edl(
    segments: &[ProcessedSegment],
    input_path: &Path,
    output_path: &Path,
    fps: f32,
) -> Result<()> {
    let filename = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("video.mp4");
    let mut edl = String::new();
    edl.push_str("TITLE: Edited Timeline\n");
    edl.push_str("FCM: NON-DROP FRAME\n\n");

    for (i, seg) in segments.iter().enumerate() {
        let (src_start_h, src_start_m, src_start_s, src_start_f) =
            seconds_to_timecode(seg.start, fps);
        let (src_end_h, src_end_m, src_end_s, src_end_f) = seconds_to_timecode(seg.end, fps);
        edl.push_str(&format!(
            "{:03}  AX       V     C        {:02}:{:02}:{:02}:{:02} {:02}:{:02}:{:02}:{:02}\n",
            i + 1,
            src_start_h,
            src_start_m,
            src_start_s,
            src_start_f,
            src_end_h,
            src_end_m,
            src_end_s,
            src_end_f
        ));
        edl.push_str(&format!("* FROM CLIP NAME: {}\n\n", filename));
    }

    fs::write(output_path, edl).context("failed to write EDL file")?;
    Ok(())
}

/// Convert seconds to SMPTE timecode (HH:MM:SS:FF)
fn seconds_to_timecode(seconds: f32, fps: f32) -> (u32, u32, u32, u32) {
    let seconds = seconds.max(0.0);
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let frames = ((seconds % 1.0) * fps).round() as u32;
    (hours, minutes, secs, frames)
}

pub fn export_srt(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut srt = String::new();
    for (i, seg) in transcript.iter().enumerate() {
        srt.push_str(&format!("{}\n", i + 1));
        srt.push_str(&format!(
            "{} --> {}\n",
            format_srt_time(seg.start),
            format_srt_time(seg.end)
        ));
        srt.push_str(&format!("{}\n\n", seg.text.trim()));
    }

    fs::write(output_path, srt).context("failed to write SRT file")?;
    Ok(())
}

pub fn export_youtube_chapters(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut chapters = String::new();
    chapters.push_str("00:00 Intro\n");

    // Group transcript segments into chapters every ~3 minutes
    // Whisper returns ~30-second chunks, so we group by ~6 segments per chapter
    let chapter_interval_secs = 180.0; // 3 minutes
    let mut chapter_start = 0.0;
    let mut chapter_texts: Vec<String> = Vec::new();

    for seg in transcript {
        if seg.start >= chapter_start + chapter_interval_secs {
            // Time to start a new chapter
            if !chapter_texts.is_empty() {
                // Use first meaningful text as chapter title (first 50 chars)
                let joined = chapter_texts.join(" ");
                let title = joined.trim();
                let title = if title.len() > 50 {
                    &title[..50]
                } else {
                    title
                };
                let title = title.replace('\n', " ").replace('\r', "");
                chapters.push_str(&format!(
                    "{} {}\n",
                    format_youtube_time(chapter_start),
                    title
                ));
            }
            chapter_start = seg.start;
            chapter_texts.clear();
        }
        // Collect non-empty text
        let text = seg.text.trim();
        if !text.is_empty() && text != "[No speech detected]" {
            chapter_texts.push(text.to_string());
        }
    }

    // Don't forget the last chapter
    if !chapter_texts.is_empty() {
        let joined = chapter_texts.join(" ");
        let title = joined.trim();
        let title = if title.len() > 50 {
            &title[..50]
        } else {
            title
        };
        let title = title.replace('\n', " ").replace('\r', "");
        chapters.push_str(&format!(
            "{} {}\n",
            format_youtube_time(chapter_start),
            title
        ));
    }

    fs::write(output_path, chapters).context("failed to write chapters file")?;
    Ok(())
}

fn format_srt_time(seconds: f32) -> String {
    // Round to nearest millisecond to avoid truncation artifacts
    let total_millis = (seconds * 1000.0).round() as u64;
    let hours = (total_millis / 3_600_000) as u32;
    let minutes = ((total_millis % 3_600_000) / 60_000) as u32;
    let secs = ((total_millis % 60_000) / 1_000) as u32;
    let millis = (total_millis % 1_000) as u32;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

fn format_youtube_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_youtube_chapters() -> Result<()> {
        let dir = tempdir()?;
        let output_chapters = dir.path().join("chapters.txt");
        // Whisper returns ~30-second chunks, so simulate a 10-minute video with multiple chunks
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 30.0,
                text: "Welcome everyone to today's video".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 30.0,
                end: 60.0,
                text: "We're going to talk about AI video editing".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 60.0,
                end: 90.0,
                text: "Let's start with the introduction".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 90.0,
                end: 120.0,
                text: "First, I'll show you the basic setup".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 200.0,
                end: 230.0,
                text: "Now let's look at the advanced features".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 230.0,
                end: 260.0,
                text: "This is where it gets really powerful".to_string(),
                confidence: 1.0,
            },
        ];

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(output_chapters)?;
        assert!(content.contains("00:00 Intro"));
        // First chapter should be at ~3 min mark (180s)
        assert!(content.contains("03:00") || content.contains("03:20"));

        Ok(())
    }

    #[test]
    fn test_export_srt() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("subtitles.srt");
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 5.0,
                text: "Hello world".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 5.0,
                end: 10.0,
                text: "This is a test".to_string(),
                confidence: 1.0,
            },
        ];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(output_srt)?;
        assert!(content.contains("1\n"));
        assert!(content.contains("Hello world"));
        assert!(content.contains("2\n"));
        assert!(content.contains("This is a test"));
        assert!(content.contains("00:00:00,000 --> 00:00:05,000"));

        Ok(())
    }

    #[test]
    fn test_export_edl_timestamps() -> Result<()> {
        let dir = tempdir()?;
        let output_edl = dir.path().join("test.edl");
        let input_path = dir.path().join("video.mp4");

        let segments = vec![
            ProcessedSegment {
                start: 0.0,
                end: 5.5,
                speed: 1.0,
            },
            ProcessedSegment {
                start: 10.0,
                end: 20.0,
                speed: 1.0,
            },
        ];

        export_edl(&segments, &input_path, &output_edl, 25.0)?;

        let content = fs::read_to_string(output_edl)?;
        // Check that timestamps are present, not all zeros
        assert!(
            content.contains("00:00:05:12") || content.contains("00:00:05:13"),
            "EDL should contain non-zero timestamps"
        );
        assert!(
            content.contains("00:00:10:00"),
            "EDL should contain second segment start"
        );
        assert!(
            content.contains("00:00:20:00"),
            "EDL should contain second segment end"
        );

        Ok(())
    }

    #[test]
    fn test_format_srt_time_rounding() -> Result<()> {
        // Test that milliseconds round properly
        let t1 = format_srt_time(1.999);
        assert_eq!(t1, "00:00:01,999");

        let t2 = format_srt_time(1.9995);
        assert_eq!(t2, "00:00:02,000");

        Ok(())
    }

    #[test]
    fn test_format_youtube_time() {
        assert_eq!(format_youtube_time(0.0), "00:00");
        assert_eq!(format_youtube_time(5.0), "00:05");
        assert_eq!(format_youtube_time(65.0), "01:05");
        assert_eq!(format_youtube_time(3661.0), "01:01:01");
        assert_eq!(format_youtube_time(3600.0), "01:00:00");
    }

    #[test]
    fn test_seconds_to_timecode() {
        let (h, m, s, f) = seconds_to_timecode(0.0, 25.0);
        assert_eq!(h, 0);
        assert_eq!(m, 0);
        assert_eq!(s, 0);
        assert_eq!(f, 0);

        let (h, m, s, f) = seconds_to_timecode(5.5, 25.0);
        assert_eq!(h, 0);
        assert_eq!(m, 0);
        assert_eq!(s, 5);
        assert_eq!(f, 13); // (5.5 % 1.0) * 25 = 0.5 * 25 = 12.5, rounded to 13

        let (h, m, s, f) = seconds_to_timecode(3661.04, 25.0);
        assert_eq!(h, 1);
        assert_eq!(m, 1);
        assert_eq!(s, 1);
        // 0.04 * 25 = 1.0, rounded to 1
        assert_eq!(f, 1);
    }
}
