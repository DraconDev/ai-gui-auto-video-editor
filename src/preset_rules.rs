use crate::config::Preset;
use std::path::Path;

/// A rule that maps filename patterns to presets
#[derive(Debug, Clone)]
pub struct PresetRule {
    /// Pattern to match (substring search, case-insensitive)
    pub pattern: String,
    /// Preset to apply when pattern matches
    pub preset: Preset,
}

impl PresetRule {
    pub fn new(pattern: impl Into<String>, preset: Preset) -> Self {
        Self {
            pattern: pattern.into().to_lowercase(),
            preset,
        }
    }
}

/// Determine the preset for a file based on filename patterns
///
/// # Arguments
/// * `path` - Path to the video file
/// * `rules` - List of preset rules to check (in order)
/// * `default_preset` - Fallback preset if no rules match
///
/// # Returns
/// The matching preset, or the default if no rules match
pub fn preset_for_file(path: &Path, rules: &[PresetRule], default_preset: Preset) -> Preset {
    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    for rule in rules {
        if filename.contains(&rule.pattern) {
            return rule.preset;
        }
    }

    default_preset
}

/// Default preset rules for common naming conventions
pub fn default_preset_rules() -> Vec<PresetRule> {
    vec![
        PresetRule::new("short", Preset::Shorts),
        PresetRule::new("tiktok", Preset::Tiktok),
        PresetRule::new("reel", Preset::Reels),
        PresetRule::new("podcast", Preset::Podcast),
        PresetRule::new("twitter", Preset::Twitter),
        PresetRule::new("x_", Preset::Twitter),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_for_file_matching() {
        let rules = vec![
            PresetRule::new("short", Preset::Shorts),
            PresetRule::new("podcast", Preset::Podcast),
        ];

        assert_eq!(
            preset_for_file(Path::new("my_short_video.mp4"), &rules, Preset::Youtube),
            Preset::Shorts
        );
        assert_eq!(
            preset_for_file(Path::new("podcast_episode_1.mp4"), &rules, Preset::Youtube),
            Preset::Podcast
        );
        assert_eq!(
            preset_for_file(Path::new("regular_video.mp4"), &rules, Preset::Youtube),
            Preset::Youtube
        );
    }

    #[test]
    fn test_preset_for_file_case_insensitive() {
        let rules = vec![PresetRule::new("SHORT", Preset::Shorts)];

        assert_eq!(
            preset_for_file(Path::new("My_Short_Video.mp4"), &rules, Preset::Youtube),
            Preset::Shorts
        );
    }

    #[test]
    fn test_preset_for_file_first_match_wins() {
        let rules = vec![
            PresetRule::new("short", Preset::Shorts),
            PresetRule::new("shorts", Preset::Tiktok), // This will never match because "short" matches first
        ];

        assert_eq!(
            preset_for_file(Path::new("shorts_video.mp4"), &rules, Preset::Youtube),
            Preset::Shorts // "short" matches before "shorts"
        );
    }

    #[test]
    fn test_default_preset_rules() {
        let rules = default_preset_rules();
        assert!(!rules.is_empty());

        assert_eq!(
            preset_for_file(Path::new("tiktok_dance.mp4"), &rules, Preset::Youtube),
            Preset::Tiktok
        );
    }

    #[test]
    fn test_preset_for_file_all_default_rules() {
        let rules = default_preset_rules();

        // Test each default rule
        assert_eq!(
            preset_for_file(Path::new("video_short.mp4"), &rules, Preset::Youtube),
            Preset::Shorts
        );
        assert_eq!(
            preset_for_file(Path::new("my_tiktok.mp4"), &rules, Preset::Youtube),
            Preset::Tiktok
        );
        assert_eq!(
            preset_for_file(Path::new("insta_reel.mp4"), &rules, Preset::Youtube),
            Preset::Reels
        );
        assert_eq!(
            preset_for_file(Path::new("podcast_audio.mp4"), &rules, Preset::Youtube),
            Preset::Podcast
        );
        assert_eq!(
            preset_for_file(Path::new("twitter_video.mp4"), &rules, Preset::Youtube),
            Preset::Twitter
        );
        assert_eq!(
            preset_for_file(Path::new("x_video.mp4"), &rules, Preset::Youtube),
            Preset::Twitter
        );
    }

    #[test]
    fn test_preset_for_file_empty_rules() {
        // With no rules, always returns default
        assert_eq!(
            preset_for_file(Path::new("anything.mp4"), &[], Preset::Minimal),
            Preset::Minimal
        );
    }

    #[test]
    fn test_preset_for_file_path_without_stem() {
        // Path with no file stem should return default
        let rules = vec![PresetRule::new("test", Preset::Shorts)];
        assert_eq!(
            preset_for_file(Path::new("/"), &rules, Preset::Youtube),
            Preset::Youtube
        );
    }

    #[test]
    fn test_preset_for_file_no_extension() {
        // File without extension - stem is the full filename
        let rules = vec![PresetRule::new("short", Preset::Shorts)];
        assert_eq!(
            preset_for_file(Path::new("my_short_video"), &rules, Preset::Youtube),
            Preset::Shorts
        );
    }

    // ── Preset matching edge cases ─────────────────────────────────────────

    #[test]
    fn test_preset_rule_exact_match() {
        let rule = PresetRule::new("podcast", Preset::Podcast);
        let rules = vec![rule];

        assert_eq!(
            preset_for_file(Path::new("my_podcast.mp3"), &rules, Preset::Minimal),
            Preset::Podcast
        );
    }

    #[test]
    fn test_preset_for_file_substring_match() {
        // "short" should match "shorts" and "short_video"
        let rules = vec![PresetRule::new("short", Preset::Shorts)];
        assert_eq!(
            preset_for_file(Path::new("my_short_video.mp4"), &rules, Preset::Minimal),
            Preset::Shorts
        );
    }

    #[test]
    fn test_preset_for_file_no_match_returns_default() {
        let rules = vec![PresetRule::new("podcast", Preset::Podcast)];
        assert_eq!(
            preset_for_file(Path::new("random_video.mp4"), &rules, Preset::Minimal),
            Preset::Minimal
        );
    }

    #[test]
    fn test_preset_for_file_empty_rules_returns_default() {
        let rules: Vec<PresetRule> = vec![];
        assert_eq!(
            preset_for_file(Path::new("anything.mp4"), &rules, Preset::Youtube),
            Preset::Youtube
        );
    }

    #[test]
    fn test_preset_rule_preserves_priority() {
        // First matching rule wins
        let rules = vec![
            PresetRule::new("interview", Preset::Podcast),
            PresetRule::new("interview", Preset::Shorts), // Same pattern, should not override
        ];
        assert_eq!(
            preset_for_file(Path::new("my_interview.mp4"), &rules, Preset::Minimal),
            Preset::Podcast
        );
    }

    #[test]
    fn test_preset_for_file_with_path() {
        let rules = vec![PresetRule::new("vlog", Preset::Minimal)];
        // Full path should still match
        assert_eq!(
            preset_for_file(
                Path::new("/home/user/videos/my_vlog.mp4"),
                &rules,
                Preset::Minimal
            ),
            Preset::Minimal
        );
    }

    // ── PresetRule edge cases ───────────────────────────────────────────────
    #[test]
    fn test_preset_for_file_multiple_patterns() {
        let rules = vec![
            PresetRule::new("podcast", Preset::Podcast),
            PresetRule::new("interview", Preset::Youtube),
            PresetRule::new("shorts", Preset::Shorts),
        ];
        // Test each pattern
        assert_eq!(
            preset_for_file(Path::new("podcast_001.mp4"), &rules, Preset::Minimal),
            Preset::Podcast
        );
        assert_eq!(
            preset_for_file(Path::new("interview_001.mp4"), &rules, Preset::Minimal),
            Preset::Youtube
        );
        assert_eq!(
            preset_for_file(Path::new("shorts_001.mp4"), &rules, Preset::Minimal),
            Preset::Shorts
        );
    }

    #[test]
    fn test_preset_for_file_no_match_fallback() {
        let rules = vec![PresetRule::new("podcast", Preset::Podcast)];
        // Filename doesn't match any rule
        let result = preset_for_file(Path::new("random_video.mp4"), &rules, Preset::Minimal);
        assert_eq!(result, Preset::Minimal);
    }

    #[test]
    fn test_preset_for_file_empty_rules_fallback() {
        let rules: Vec<PresetRule> = vec![];
        let result = preset_for_file(Path::new("any_video.mp4"), &rules, Preset::Twitter);
        assert_eq!(result, Preset::Twitter);
    }

    #[test]
    fn test_preset_rule_new() {
        let rule = PresetRule::new("test", Preset::Podcast);
        assert_eq!(rule.pattern, "test");
        assert!(matches!(rule.preset, Preset::Podcast));
    }
}
