use serde::{Deserialize, Serialize};
use tench_document_core::OfficeFileFormat;
use tench_office_io::config_io;

const PRODUCT_ID: &str = "tench-sheets";
const CONFIG_FILENAME: &str = "config.json";

/// Grid display settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GridSettings {
    /// Default number of rows.
    pub default_rows: u32,
    /// Default number of columns.
    pub default_columns: u32,
    /// Default row height in pixels.
    pub row_height: u32,
    /// Default column width in pixels.
    pub column_width: u32,
    /// Show gridlines.
    pub show_gridlines: bool,
    /// Show row/column headers.
    pub show_headers: bool,
    /// Default zoom percentage (50-200).
    pub default_zoom: u32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            default_rows: 1000,
            default_columns: 26,
            row_height: 24,
            column_width: 80,
            show_gridlines: true,
            show_headers: true,
            default_zoom: 100,
        }
    }
}

/// Formula engine settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FormulaSettings {
    /// Enable iterative calculation.
    pub iterative_calculation: bool,
    /// Maximum iterations.
    pub max_iterations: u32,
    /// Calculation precision.
    pub precision: u32,
}

impl Default for FormulaSettings {
    fn default() -> Self {
        Self {
            iterative_calculation: false,
            max_iterations: 100,
            precision: 15,
        }
    }
}

/// AI assistant settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AiSettings {
    pub engine_url: String,
    pub enable_suggestions: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            engine_url: "http://localhost:8080".to_string(),
            enable_suggestions: true,
        }
    }
}

/// Save and backup settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveSettings {
    pub default_format: OfficeFileFormat,
    pub max_backups: u32,
    pub enable_recovery: bool,
    pub autosave_interval_secs: u32,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            default_format: OfficeFileFormat::Xlsx,
            max_backups: 10,
            enable_recovery: true,
            autosave_interval_secs: 30,
        }
    }
}

/// Complete Sheets application configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SheetsConfig {
    #[serde(default)]
    pub grid: GridSettings,
    #[serde(default)]
    pub formula: FormulaSettings,
    #[serde(default)]
    pub ai: AiSettings,
    #[serde(default)]
    pub save: SaveSettings,
}

pub fn load_config() -> Result<SheetsConfig, String> {
    config_io::load_json_config(PRODUCT_ID, CONFIG_FILENAME, SheetsConfig::default())
}

pub fn save_config(config: &SheetsConfig) -> Result<(), String> {
    config_io::save_json_config(PRODUCT_ID, CONFIG_FILENAME, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = SheetsConfig::default();
        assert_eq!(config.grid.default_rows, 1000);
        assert_eq!(config.grid.default_columns, 26);
        assert_eq!(config.formula.max_iterations, 100);
        assert_eq!(config.save.default_format, OfficeFileFormat::Xlsx);
    }

    #[test]
    fn config_round_trips_through_json() {
        let config = SheetsConfig {
            grid: GridSettings {
                default_rows: 500,
                column_width: 100,
                ..GridSettings::default()
            },
            ..SheetsConfig::default()
        };
        let json = serde_json::to_string_pretty(&config).expect("serialize");
        let parsed: SheetsConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.grid.default_rows, 500);
        assert_eq!(parsed.grid.column_width, 100);
    }

    #[test]
    fn load_config_returns_default_when_missing() {
        let _guard = isolated_storage("missing");
        let config = load_config().expect("load config");

        assert_eq!(config.grid.default_rows, 1000);
        assert_eq!(config.save.default_format, OfficeFileFormat::Xlsx);
    }

    #[test]
    fn save_and_load_config_round_trips() {
        let _guard = isolated_storage("roundtrip");
        let mut config = SheetsConfig::default();
        config.grid.default_rows = 500;
        config.save.default_format = OfficeFileFormat::Csv;

        save_config(&config).expect("save config");
        let loaded = load_config().expect("load config");

        assert_eq!(loaded.grid.default_rows, 500);
        assert_eq!(loaded.save.default_format, OfficeFileFormat::Csv);
    }

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_sheets_config_{name}_{}_{}",
            std::process::id(),
            tench_office_io::file_util::timestamp_millis()
        ));
        crate::test_util::set_test_data_home(&dir);
        std::fs::create_dir_all(&dir).expect("test dir");
        guard
    }
}
