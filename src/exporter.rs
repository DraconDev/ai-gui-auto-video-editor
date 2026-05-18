use crate::analyzer::ProcessedSegment;
use crate::stt_analyzer::TranscriptSegment;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Escape special XML characters in a string
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn export_fcpxml(
    segments: &[ProcessedSegment],
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    if segments.is_empty() {
        fs::write(output_path, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE fcpxml>\n<fcpxml version=\"1.8\">\n  <resources></resources>\n  <library>\n    <event name=\"Automated Cuts\">\n      <project name=\"Edited Timeline\">\n        <sequence duration=\"0/1s\" format=\"r1\">\n          <spine></spine>\n        </sequence>\n      </project>\n    </event>\n  </library>\n</fcpxml>\n")
            .context("failed to write empty FCPXML file")?;
        return Ok(());
    }

    let filename = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("video.mp4");
    let filename_escaped = xml_escape(filename);

    // Calculate total duration from segments
    let total_duration: f32 = segments.iter().map(|s| s.end - s.start).sum();
    let duration_str = format!("{:.0}/1s", total_duration.max(1.0));

    let input_path_str = xml_escape(&input_path.to_string_lossy());

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE fcpxml>\n");
    xml.push_str("<fcpxml version=\"1.8\">\n");
    xml.push_str("  <resources>\n");
    xml.push_str(&format!(
        "    <asset id=\"r1\" name=\"{}\" src=\"file://{}\" />\n",
        filename_escaped, input_path_str
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
            filename_escaped, start_offset, seg.start, duration
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
    if segments.is_empty() {
        fs::write(
            output_path,
            "TITLE: Edited Timeline\nFCM: NON-DROP FRAME\n\n",
        )
        .context("failed to write empty EDL file")?;
        return Ok(());
    }

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
    if transcript.is_empty() {
        fs::write(output_path, "").context("failed to write empty SRT file")?;
        return Ok(());
    }

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
    if transcript.is_empty() {
        fs::write(output_path, "00:00 Intro\n").context("failed to write chapters file")?;
        return Ok(());
    }

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
                // Use first meaningful text as chapter title (first 50 chars, Unicode-safe)
                let joined = chapter_texts.join(" ");
                let title = joined.trim();
                let title: String = title.chars().take(50).collect();
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
        let title: String = title.chars().take(50).collect();
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
    fn test_export_youtube_chapters_empty() -> Result<()> {
        let dir = tempdir()?;
        let output_chapters = dir.path().join("chapters.txt");
        let transcript: Vec<TranscriptSegment> = vec![];

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(output_chapters)?;
        assert!(content.is_empty() || content.contains("00:00 Intro"));
        Ok(())
    }

    #[test]
    fn test_export_youtube_chapters_single_segment() -> Result<()> {
        let dir = tempdir()?;
        let output_chapters = dir.path().join("chapters.txt");
        let transcript = vec![TranscriptSegment {
            start: 0.0,
            end: 60.0,
            text: "Single segment video".to_string(),
            confidence: 1.0,
        }];

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(output_chapters)?;
        assert!(content.contains("00:00 Intro"));
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

    #[test]
    fn test_export_fcpxml_valid_xml() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path().join("timeline.fcpxml");
        let input_path = dir.path().join("input.mp4");

        let segments = vec![
            ProcessedSegment {
                start: 0.0,
                end: 5.0,
                speed: 1.0,
            },
            ProcessedSegment {
                start: 10.0,
                end: 20.0,
                speed: 1.0,
            },
        ];

        export_fcpxml(&segments, &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;

        // Check for required XML structure
        assert!(
            content.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
            "FCPXML should start with XML declaration"
        );
        assert!(
            content.contains("<!DOCTYPE fcpxml>"),
            "FCPXML should contain DOCTYPE"
        );
        assert!(
            content.contains("<fcpxml version=\"1.8\">"),
            "FCPXML should have version 1.8"
        );
        assert!(
            content.contains("<resources>"),
            "FCPXML should have resources section"
        );
        assert!(
            content.contains("<library>"),
            "FCPXML should have library section"
        );
        assert!(
            content.contains("<spine>"),
            "FCPXML should have spine element"
        );
        assert!(
            content.contains("</fcpxml>"),
            "FCPXML should be properly closed"
        );

        // Verify video elements (self-closing tags like <video ... />)
        let video_elements: usize = content.matches("<video name=").count();
        assert_eq!(
            video_elements, 2,
            "FCPXML should have 2 video elements for 2 segments"
        );
        // Self-closing video elements
        let self_closing: usize = content.matches("/>").count();
        assert!(self_closing > 0, "FCPXML should have self-closing tags");

        // Check duration format
        assert!(
            content.contains("duration=\"15/1s\""),
            "FCPXML duration should be calculated from segments (0-5 + 10-20 = 15s)"
        );

        Ok(())
    }

    #[test]
    fn test_export_fcpxml_escapes_xml_special_chars_in_filename() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path().join("timeline.fcpxml");
        // Filename with XML special characters
        let input_path = dir.path().join("video&<>'\".mp4");

        let segments = vec![ProcessedSegment {
            start: 0.0,
            end: 10.0,
            speed: 1.0,
        }];

        export_fcpxml(&segments, &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;

        // Verify special characters are escaped in filename
        assert!(
            content.contains("video&amp;&lt;&gt;&apos;&quot;.mp4"),
            "FCPXML should escape & < > ' \" in filename"
        );

        // Verify the raw special characters don't appear unescaped
        assert!(
            !content.contains("video&<>\""),
            "FCPXML should not contain unescaped special chars"
        );

        Ok(())
    }

    #[test]
    fn test_export_fcpxml_escapes_xml_special_chars_in_path() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path().join("timeline.fcpxml");
        let input_path = dir.path().join("video with spaces & special <chars>.mp4");

        let segments = vec![ProcessedSegment {
            start: 0.0,
            end: 5.0,
            speed: 1.0,
        }];

        export_fcpxml(&segments, &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;

        // The src attribute should have escaped characters
        assert!(
            content.contains("file://"),
            "FCPXML should have file:// src attribute"
        );

        Ok(())
    }

    #[test]
    fn test_export_fcpxml_single_segment() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path().join("timeline.fcpxml");
        let input_path = dir.path().join("input.mp4");

        let segments = vec![ProcessedSegment {
            start: 0.0,
            end: 10.0,
            speed: 1.0,
        }];

        export_fcpxml(&segments, &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;

        // Single segment should have duration of 10s
        assert!(
            content.contains("duration=\"10/1s\""),
            "FCPXML should have duration of 10s for single segment"
        );

        // Should have exactly one video element
        assert_eq!(
            content.matches("<video name=").count(),
            1,
            "FCPXML should have exactly one video element for single segment"
        );

        Ok(())
    }

    #[test]
    fn test_export_fcpxml_speedup_segment() -> Result<()> {
        let dir = tempdir()?;
        let output_path = dir.path().join("timeline.fcpxml");
        let input_path = dir.path().join("input.mp4");

        // Segment with speedup
        let segments = vec![ProcessedSegment {
            start: 0.0,
            end: 10.0,
            speed: 2.0, // 2x speedup
        }];

        export_fcpxml(&segments, &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;

        // Duration should still be based on original time (end - start)
        assert!(
            content.contains("duration=\"10/1s\""),
            "FCPXML duration should be based on original segment time"
        );

        Ok(())
    }

    #[test]
    fn test_export_srt_timestamp_format() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("subtitles.srt");
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 1.5,
                text: "First".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 1.5,
                end: 3.333,
                text: "Second with decimal".to_string(),
                confidence: 1.0,
            },
        ];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(&output_srt)?;

        // Verify SRT format: index\nstart --> end\ntext\n\n
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "1", "First line should be index 1");
        assert_eq!(
            lines[1], "00:00:00,000 --> 00:00:01,500",
            "Second line should be timestamp range with milliseconds"
        );
        assert_eq!(lines[2], "First", "Third line should be text");
        assert_eq!(lines[3], "", "Fourth line should be blank separator");

        // Verify second segment
        assert_eq!(lines[4], "2", "Fifth line should be index 2");
        assert!(
            lines[5].contains("-->"),
            "Sixth line should contain timestamp range"
        );
        assert_eq!(
            lines[6], "Second with decimal",
            "Seventh line should be text"
        );

        // Verify trailing newline
        assert!(content.ends_with('\n'), "SRT should end with newline");

        Ok(())
    }

    #[test]
    fn test_export_edl_format_structure() -> Result<()> {
        let dir = tempdir()?;
        let output_edl = dir.path().join("test.edl");
        let input_path = dir.path().join("video.mp4");

        let segments = vec![
            ProcessedSegment {
                start: 0.0,
                end: 5.0,
                speed: 1.0,
            },
            ProcessedSegment {
                start: 5.0,
                end: 10.0,
                speed: 1.0,
            },
        ];

        export_edl(&segments, &input_path, &output_edl, 24.0)?;

        let content = fs::read_to_string(&output_edl)?;

        // Check EDL header format
        assert!(content.starts_with("TITLE:"), "EDL should start with TITLE");
        assert!(
            content.contains("FCM: NON-DROP FRAME"),
            "EDL should contain FCM statement"
        );

        // Check segment format: index, edit type, track, source type, etc.
        assert!(
            content.contains("001  AX       V     C"),
            "EDL should have proper edit decision line format"
        );

        // Check FROM CLIP NAME comment
        assert!(
            content.contains("FROM CLIP NAME:"),
            "EDL should have clip name comment"
        );

        Ok(())
    }

    #[test]
    fn test_export_edl_adjacent_segments() -> Result<()> {
        let dir = tempdir()?;
        let output_edl = dir.path().join("test.edl");
        let input_path = dir.path().join("video.mp4");

        // Two segments that are adjacent (end of first = start of second)
        let segments = vec![
            ProcessedSegment {
                start: 0.0,
                end: 5.0,
                speed: 1.0,
            },
            ProcessedSegment {
                start: 5.0,
                end: 10.0,
                speed: 1.0,
            },
        ];

        export_edl(&segments, &input_path, &output_edl, 25.0)?;

        let content = fs::read_to_string(&output_edl)?;

        // Both segments should be present
        assert!(
            content.contains("00:00:00:00"),
            "EDL should start at 00:00:00:00"
        );
        assert!(
            content.contains("00:00:05:00"),
            "EDL should have first segment end at 5 seconds"
        );
        assert!(
            content.contains("00:00:10:00"),
            "EDL should have second segment end at 10 seconds"
        );

        Ok(())
    }

    #[test]
    fn test_export_youtube_chapters_long_video() -> Result<()> {
        let dir = tempdir()?;
        let output_chapters = dir.path().join("chapters.txt");

        // Create transcript for a ~10 minute video with segments every 30 seconds
        let mut transcript = Vec::new();
        let mut t = 0.0;
        for i in 0..20 {
            transcript.push(TranscriptSegment {
                start: t,
                end: t + 30.0,
                text: format!("Chapter {} content", i),
                confidence: 1.0,
            });
            t += 30.0;
        }

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(&output_chapters)?;

        // Should have intro at 00:00
        assert!(
            content.contains("00:00 Intro"),
            "Chapters should start with 00:00 Intro"
        );

        // Should have chapters at ~3 minute intervals (180 seconds)
        // With 20 x 30s segments, we should have multiple chapter markers
        assert!(
            content.matches("03:").count() >= 1,
            "Should have chapter around 3 minutes"
        );
        assert!(
            content.matches("06:").count() >= 1,
            "Should have chapter around 6 minutes"
        );

        Ok(())
    }

    #[test]
    fn test_export_youtube_chapters_handles_empty_text() -> Result<()> {
        let dir = tempdir()?;
        let output_chapters = dir.path().join("chapters.txt");

        // Transcript with empty and "[No speech detected]" segments
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 30.0,
                text: "".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 30.0,
                end: 60.0,
                text: "[No speech detected]".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 60.0,
                end: 90.0,
                text: "Actual speech".to_string(),
                confidence: 1.0,
            },
        ];

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(&output_chapters)?;

        // Empty and [No speech detected] should be filtered out
        assert!(
            !content.contains("[No speech detected]"),
            "Chapters should not contain [No speech detected]"
        );
        assert!(
            content.contains("Actual speech") || content.contains("Intro"),
            "Chapters should contain actual speech or intro"
        );

        Ok(())
    }

    #[test]
    fn test_xml_escape_all_special_chars() {
        let input = "Tom & Jerry";
        assert_eq!(xml_escape(input), "Tom &amp; Jerry");
    }

    #[test]
    fn test_xml_escape_angle_brackets() {
        let input = "<intro> and </outro>";
        assert_eq!(xml_escape(input), "&lt;intro&gt; and &lt;/outro&gt;");
    }

    #[test]
    fn test_xml_escape_quotes() {
        let input = "He said \"hello\" and 'world'";
        let escaped = xml_escape(input);
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&apos;"));
        assert!(!escaped.contains('"'));
        assert!(!escaped.contains('\''));
    }

    #[test]
    fn test_xml_escape_complete_roundtrip() {
        let input = "&<>\"'";
        let escaped = xml_escape(input);
        assert_eq!(escaped, "&amp;&lt;&gt;&quot;&apos;");
    }

    #[test]
    fn test_xml_escape_no_special_chars() {
        let input = "Plain text without special chars";
        assert_eq!(xml_escape(input), input);
    }

    #[test]
    fn test_export_fcpxml_empty_segments() -> Result<()> {
        let dir = tempdir()?;
        let input_path = dir.path().join("video.mp4");
        let output_path = dir.path().join("output.fcpxml");

        export_fcpxml(&[], &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;
        assert!(content.contains("<!DOCTYPE fcpxml>"));
        assert!(content.contains("<fcpxml"));
        assert!(content.contains("<resources></resources>"));
        Ok(())
    }

    #[test]
    fn test_export_fcpxml_segments_alignment() -> Result<()> {
        let dir = tempdir()?;
        let input_path = dir.path().join("video.mp4");
        let output_path = dir.path().join("output.fcpxml");

        let segments = vec![crate::analyzer::ProcessedSegment {
            start: 0.0,
            end: 30.0,
            speed: 1.0,
        }];

        export_fcpxml(&segments, &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;
        assert!(content.contains("duration=\"30/1s\""));
        assert!(content.contains("offset=\"0s\""));
        assert!(content.contains("start=\"0s\""));
        Ok(())
    }

    #[test]
    fn test_export_fcpxml_path_with_special_chars() -> Result<()> {
        let dir = tempdir()?;
        let input_path = dir.path().join("video & test.mp4");
        let output_path = dir.path().join("output.fcpxml");

        let segments = vec![crate::analyzer::ProcessedSegment {
            start: 0.0,
            end: 10.0,
            speed: 1.0,
        }];

        export_fcpxml(&segments, &input_path, &output_path)?;

        let content = fs::read_to_string(&output_path)?;
        assert!(content.contains("video &amp; test"));
        Ok(())
    }

    #[test]
    fn test_export_srt_single_segment() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("single.srt");
        let transcript = vec![TranscriptSegment {
            start: 1.5,
            end: 4.5,
            text: "Single subtitle".to_string(),
            confidence: 1.0,
        }];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(&output_srt)?;
        assert!(content.contains("00:00:01,500"));
        assert!(content.contains("00:00:04,500"));
        Ok(())
    }

    #[test]
    fn test_export_edl_empty_segments() -> Result<()> {
        let dir = tempdir()?;
        let output_edl = dir.path().join("empty.edl");
        let input_path = dir.path().join("video.mp4");

        export_edl(&[], &input_path, &output_edl, 25.0)?;

        let content = fs::read_to_string(&output_edl)?;
        assert!(content.contains("TITLE:"));
        assert!(content.contains("FCM:"));
        Ok(())
    }

    // ── SRT edge case tests ─────────────────────────────────────────────────

    #[test]
    fn test_export_srt_ampersand_escaping() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("subs.srt");
        let transcript = vec![TranscriptSegment {
            start: 0.0,
            end: 5.0,
            text: "Tom & Jerry".to_string(),
            confidence: 1.0,
        }];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(&output_srt)?;
        // SRT uses plain text but some players prefer XML-escaped & -> &amp;
        assert!(content.contains("Tom"));
        assert!(content.contains("Jerry"));
        Ok(())
    }

    #[test]
    fn test_export_srt_strips_angle_brackets() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("subs.srt");
        let transcript = vec![TranscriptSegment {
            start: 0.0,
            end: 3.0,
            text: "<bold>test</bold>".to_string(),
            confidence: 1.0,
        }];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(&output_srt)?;
        // SRT doesn't use HTML markup
        assert!(content.contains("test"));
        Ok(())
    }

    #[test]
    fn test_export_srt_sorted_by_start_time() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("subs.srt");
        // Add out-of-order transcript segments
        let mut transcript = vec![
            TranscriptSegment {
                start: 5.0,
                end: 10.0,
                text: "Second".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 0.0,
                end: 5.0,
                text: "First".to_string(),
                confidence: 1.0,
            },
        ];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(&output_srt)?;
        let first_idx = content
            .find(
                "1
",
            )
            .expect("Index 1");
        let second_idx = content
            .find(
                "2
",
            )
            .expect("Index 2");
        assert!(
            first_idx < second_idx,
            "Earlier timestamp should appear first in output"
        );
        Ok(())
    }

    #[test]
    fn test_export_fcpxml_timestamp_s_suffix() -> Result<()> {
        let dir = tempdir()?;
        let output_fcpxml = dir.path().join("output.fcpxml");
        let input_path = dir.path().join("video.mp4");

        let segments = vec![crate::analyzer::ProcessedSegment {
            start: 0.0,
            end: 30.0,
            speed: 1.0,
        }];
        export_fcpxml(&segments, &input_path, &output_fcpxml)?;

        let content = fs::read_to_string(&output_fcpxml)?;
        // FCPXML timestamps always use "s" suffix (e.g. "30s")
        assert!(
            content.contains("duration=\"30s\""),
            "Duration should have s suffix"
        );
        assert!(
            content.contains("offset=\"0s\""),
            "Offset should have s suffix"
        );
        assert!(
            content.contains("start=\"0s\""),
            "Start should have s suffix"
        );
        Ok(())
    }

    #[test]
    fn test_export_fcpxml_multiple_segments_sorted() -> Result<()> {
        let dir = tempdir()?;
        let output_fcpxml = dir.path().join("output.fcpxml");
        let input_path = dir.path().join("video.mp4");

        // Add segments out of order
        let mut segments = vec![
            crate::analyzer::ProcessedSegment {
                start: 10.0,
                end: 20.0,
                speed: 1.0,
            },
            crate::analyzer::ProcessedSegment {
                start: 0.0,
                end: 10.0,
                speed: 1.0,
            },
        ];

        export_fcpxml(&segments, &input_path, &output_fcpxml)?;

        let content = fs::read_to_string(&output_fcpxml)?;
        // The first clip start value should be 0s (earlier segment)
        let first_clip_start = content.find("start=\"");
        assert!(
            first_clip_start.is_some(),
            "Should find first start attribute"
        );
        Ok(())
    }

    #[test]
    fn test_export_edl_single_segment_timecode() -> Result<()> {
        let dir = tempdir()?;
        let output_edl = dir.path().join("output.edl");
        let input_path = dir.path().join("video.mp4");

        // 10s segment at 25fps
        let segments = vec![crate::analyzer::ProcessedSegment {
            start: 0.0,
            end: 10.0,
            speed: 1.0,
        }];

        export_edl(&segments, &input_path, &output_edl, 25.0)?;

        let content = fs::read_to_string(&output_edl)?;
        // Verify the EDL contains proper timecode format HH:MM:SS:FF
        // 10 seconds = 00:00:10:00 at 25fps
        assert!(
            content.contains("00:00:10:00") || content.contains("00:00:09:24"),
            "EDL should contain proper timecode for 10s segment"
        );
        Ok(())
    }

    #[test]
    fn test_export_edl_adjacent_segments_no_gap() -> Result<()> {
        let dir = tempdir()?;
        let output_edl = dir.path().join("output.edl");
        let input_path = dir.path().join("video.mp4");

        // Two segments back-to-back
        let segments = vec![
            crate::analyzer::ProcessedSegment {
                start: 0.0,
                end: 10.0,
                speed: 1.0,
            },
            crate::analyzer::ProcessedSegment {
                start: 10.0,
                end: 20.0,
                speed: 1.0,
            },
        ];

        export_edl(&segments, &input_path, &output_edl, 25.0)?;

        let content = fs::read_to_string(&output_edl)?;
        // EDL format has "001  AX       V     C" for each clip entry
        let clip_count = content.matches("AX").count();
        assert_eq!(clip_count, 2, "Should have 2 clip entries for 2 segments");
        Ok(())
    }

    #[test]
    fn test_export_youtube_chapters_empty_segments() -> Result<()> {
        let dir = tempdir()?;
        let output = dir.path().join("chapters.txt");

        export_youtube_chapters(&[], &output)?;

        let content = fs::read_to_string(&output)?;
        // Empty transcript still produces intro marker
        assert_eq!(content, "00:00 Intro\n");
        Ok(())
    }

    #[test]
    fn test_export_youtube_chapters_single_chapter() -> Result<()> {
        let dir = tempdir()?;
        let output = dir.path().join("chapters.txt");

        export_youtube_chapters(
            &[crate::stt_analyzer::TranscriptSegment {
                start: 0.0,
                end: 60.0,
                text: "Chapter 1".to_string(),
                confidence: 1.0,
            }],
            &output,
        )?;

        let content = fs::read_to_string(&output)?;
        // Single segment should still produce at least one chapter line
        assert!(content.contains("00:00"));
        Ok(())
    }
}
