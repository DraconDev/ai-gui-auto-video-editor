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
    use std::path::PathBuf;

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
}
