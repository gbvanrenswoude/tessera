//! User preferences, stored as JSON next to the disk index.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    Mars,
    LiquidGlass,
    Minimal,
    #[default]
    #[serde(other)]
    Default,
}

impl ThemePreset {
    pub const ALL: [Self; 4] = [Self::Default, Self::Mars, Self::LiquidGlass, Self::Minimal];

    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Mars => "Mars",
            Self::LiquidGlass => "Liquid Glass",
            Self::Minimal => "Minimal",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: ThemePreset,
    /// Show the live CPU percentage next to the menu bar icon.
    pub cpu_in_menu_bar: bool,
    /// Extra folders to leave out of disk scans.
    pub excluded: Vec<String>,
    /// Re-scan automatically when the index is older than this many hours.
    pub rescan_hours: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemePreset::Default,
            cpu_in_menu_bar: false,
            excluded: Vec::new(),
            rescan_hours: 6,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read(tessera_core::paths::settings_file())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = tessera_core::paths::settings_file();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_vec_pretty(self) {
            let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
            if let Err(err) = std::fs::write(&tmp, json).and_then(|()| std::fs::rename(&tmp, &path))
            {
                eprintln!("tessera: could not save preferences: {err}");
                let _ = std::fs::remove_file(tmp);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_preferences_survive_a_theme_change() {
        let old = r#"{"cpu_in_menu_bar":true,"excluded":["/scratch"],"rescan_hours":12}"#;
        let mut settings: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(settings.theme, ThemePreset::Default);
        settings.theme = ThemePreset::LiquidGlass;
        let saved = serde_json::to_vec(&settings).unwrap();
        let restored: Settings = serde_json::from_slice(&saved).unwrap();
        assert_eq!(restored.theme, ThemePreset::LiquidGlass);
        assert!(restored.cpu_in_menu_bar);
        assert_eq!(restored.excluded, ["/scratch"]);
        assert_eq!(restored.rescan_hours, 12);
    }
}
