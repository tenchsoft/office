use serde::{Deserialize, Serialize};
use tench_document_core::OfficeFileFormat;
use tench_office_io::config_io;

const PRODUCT_ID: &str = "tench-slides";
const CONFIG_FILENAME: &str = "config.json";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SlidesConfig {
    #[serde(default)]
    pub editor: EditorSettings,
    #[serde(default)]
    pub ai: AiSettings,
    #[serde(default)]
    pub save: SaveSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorSettings {
    pub show_grid: bool,
    pub show_rulers: bool,
    pub snap_to_grid: bool,
    pub default_slide_size: String,
    pub auto_save_interval_secs: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiSettings {
    pub enabled: bool,
    pub auto_suggest: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveSettings {
    pub default_format: OfficeFileFormat,
    pub create_backup: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_rulers: true,
            snap_to_grid: true,
            default_slide_size: "16:9".to_string(),
            auto_save_interval_secs: 30,
        }
    }
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_suggest: false,
        }
    }
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            default_format: OfficeFileFormat::Pptx,
            create_backup: true,
        }
    }
}

pub fn load_config() -> Result<SlidesConfig, String> {
    config_io::load_json_config(PRODUCT_ID, CONFIG_FILENAME, SlidesConfig::default())
}

pub fn save_config(config: &SlidesConfig) -> Result<(), String> {
    config_io::save_json_config(PRODUCT_ID, CONFIG_FILENAME, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config_returns_default_when_missing() {
        let _guard = isolated_storage("missing");
        let config = load_config().expect("load config");

        assert!(config.editor.show_grid);
        assert_eq!(config.save.default_format, OfficeFileFormat::Pptx);
    }

    #[test]
    fn save_and_load_config_round_trips() {
        let _guard = isolated_storage("roundtrip");
        let mut config = SlidesConfig::default();
        config.editor.default_slide_size = "4:3".to_string();
        config.save.default_format = OfficeFileFormat::Odp;

        save_config(&config).expect("save config");
        let loaded = load_config().expect("load config");

        assert_eq!(loaded.editor.default_slide_size, "4:3");
        assert_eq!(loaded.save.default_format, OfficeFileFormat::Odp);
    }

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_slides_config_{name}_{}_{}",
            std::process::id(),
            tench_office_io::file_util::timestamp_millis()
        ));
        crate::test_util::set_test_data_home(&dir);
        std::fs::create_dir_all(&dir).expect("test dir");
        guard
    }
}
