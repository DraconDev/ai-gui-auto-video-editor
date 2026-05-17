use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

/// Benchmark the silence detection parser with a realistic FFmpeg output
fn bench_parse_ffmpeg_silence(c: &mut Criterion) {
    // Build a realistic output with 50 silence segments in mixed noise
    let mut output = String::new();
    for i in 0..50 {
        let start = i as f32 * 10.0 + 1.0;
        let end = start + 3.0;
        output.push_str(&format!(
            "[silencedetect @ 0x559e1c2c4840] silence_start: {start}\n\
             [silencedetect @ 0x559e1c2c4840] silence_end: {end} | silence_duration: {}\n",
            end - start
        ));
    }
    // Add some noise lines
    output.push_str("random noise line\ntimestamps and other ffmpeg output\n");

    c.bench_function("parse_ffmpeg_silence_50_segments", |b| {
        b.iter(|| {
            let result = ai_vid_editor::analyzer::parse_ffmpeg_silence(black_box(&output), 500.0);
            black_box(result)
        });
    });
}

criterion_group!(
    name = parsers;
    config = Criterion::default().measurement_time(Duration::from_secs(3)).warm_up_time(Duration::from_secs(1));
    targets = bench_parse_ffmpeg_silence
);
criterion_main!(parsers);
