use serde::{Deserialize, Serialize};
use tench_document_core::OfficeFileFormat;
use tench_office_io::config_io;

const PRODUCT_ID: &str = "tench-kodocs";
const CONFIG_FILENAME: &str = "config.json";

/// Editor display settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    /// Font family for the editor.
    pub font_family: String,
    /// Font size in pixels.
    pub font_size: u32,
    /// Auto-save interval in seconds (0 = disabled).
    pub autosave_interval_secs: u32,
    /// Show ruler by default.
    pub show_ruler: bool,
    /// Default view mode: "page" or "web".
    pub default_view: String,
    /// Default zoom percentage (50-200).
    pub default_zoom: u32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_family: "Malgun Gothic".to_string(),
            font_size: 16,
            autosave_interval_secs: 30,
            show_ruler: false,
            default_view: "web".to_string(),
            default_zoom: 100,
        }
    }
}

/// AI assistant settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AiSettings {
    /// Engine URL (e.g. "http://localhost:8080").
    pub engine_url: String,
    /// Tone for AI suggestions.
    pub tone: String,
    /// Enable AI ghost text suggestions.
    pub enable_suggestions: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            engine_url: "http://localhost:8080".to_string(),
            tone: "professional".to_string(),
            enable_suggestions: true,
        }
    }
}

/// Save and backup settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveSettings {
    /// Default export format.
    pub default_format: OfficeFileFormat,
    /// Maximum number of backup copies to keep.
    pub max_backups: u32,
    /// Create recovery snapshots (auto-save).
    pub enable_recovery: bool,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            default_format: OfficeFileFormat::Hwpx,
            max_backups: 10,
            enable_recovery: true,
        }
    }
}

/// Complete Docs application configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KodocsConfig {
    #[serde(default)]
    pub editor: EditorSettings,
    #[serde(default)]
    pub ai: AiSettings,
    #[serde(default)]
    pub save: SaveSettings,
}

/// Load configuration from disk. Returns defaults if no config file exists.
pub fn load_config() -> Result<KodocsConfig, String> {
    config_io::load_json_config(PRODUCT_ID, CONFIG_FILENAME, KodocsConfig::default())
}

/// Save configuration to disk.
pub fn save_config(config: &KodocsConfig) -> Result<(), String> {
    config_io::save_json_config(PRODUCT_ID, CONFIG_FILENAME, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = KodocsConfig::default();
        assert_eq!(config.editor.font_size, 16);
        assert_eq!(config.editor.autosave_interval_secs, 30);
        assert_eq!(config.ai.engine_url, "http://localhost:8080");
        assert_eq!(config.save.max_backups, 10);
    }

    #[test]
    fn config_round_trips_through_json() {
        let config = KodocsConfig {
            editor: EditorSettings {
                font_family: "monospace".to_string(),
                font_size: 14,
                autosave_interval_secs: 60,
                show_ruler: true,
                default_view: "page".to_string(),
                default_zoom: 120,
            },
            ai: AiSettings::default(),
            save: SaveSettings {
                default_format: OfficeFileFormat::Md,
                max_backups: 5,
                enable_recovery: false,
            },
        };
        let json = serde_json::to_string_pretty(&config).expect("serialize");
        let parsed: KodocsConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.editor.font_family, "monospace");
        assert_eq!(parsed.editor.font_size, 14);
        assert_eq!(parsed.save.default_format, OfficeFileFormat::Md);
        assert!(!parsed.save.enable_recovery);
    }

    #[test]
    fn config_deserialize_with_missing_fields_uses_defaults() {
        let json = r#"{"editor": {"font_family": "serif"}}"#;
        let config: KodocsConfig = serde_json::from_str(json).expect("partial config");
        assert_eq!(config.editor.font_family, "serif");
        // Other fields should be default
        assert_eq!(config.ai.engine_url, "http://localhost:8080");
        assert_eq!(config.save.max_backups, 10);
    }

    #[test]
    fn load_config_returns_default_when_missing() {
        let _guard = isolated_storage("missing");
        let config = load_config().expect("load config");

        assert_eq!(config.editor.font_size, 16);
        assert_eq!(config.save.default_format, OfficeFileFormat::Hwpx);
    }

    #[test]
    fn save_and_load_config_round_trips() {
        let _guard = isolated_storage("roundtrip");
        let mut config = KodocsConfig::default();
        config.editor.font_family = "monospace".to_string();
        config.save.default_format = OfficeFileFormat::Md;

        save_config(&config).expect("save config");
        let loaded = load_config().expect("load config");

        assert_eq!(loaded.editor.font_family, "monospace");
        assert_eq!(loaded.save.default_format, OfficeFileFormat::Md);
    }

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_docs_config_{name}_{}_{}",
            std::process::id(),
            tench_office_io::file_util::timestamp_millis()
        ));
        crate::test_util::set_test_data_home(&dir);
        std::fs::create_dir_all(&dir).expect("test dir");
        guard
    }
}
