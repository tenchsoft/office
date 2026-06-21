use std::fs;

use serde::{de::DeserializeOwned, Serialize};
use tench_storage_core::{office_storage_dir, OfficeStorageArea};

pub fn load_json_config<T>(product_id: &str, filename: &str, default: T) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let path = config_path(product_id, filename)?;
    if !path.exists() {
        return Ok(default);
    }

    let raw =
        fs::read_to_string(&path).map_err(|error| format!("Failed to read config: {error}"))?;
    serde_json::from_str(&raw).map_err(|error| format!("Failed to parse config: {error}"))
}

pub fn save_json_config<T>(product_id: &str, filename: &str, config: &T) -> Result<(), String>
where
    T: Serialize,
{
    let path = config_path(product_id, filename)?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Failed to serialize config: {error}"))?;
    fs::write(&path, json).map_err(|error| format!("Failed to write config: {error}"))
}

fn config_path(product_id: &str, filename: &str) -> Result<std::path::PathBuf, String> {
    let dir = office_storage_dir(product_id, OfficeStorageArea::Config);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create config directory: {error}"))?;
    Ok(dir.join(filename))
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        value: String,
    }

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_office_config_{name}_{}_{}",
            std::process::id(),
            crate::file_util::timestamp_millis()
        ));
        crate::test_util::set_test_data_home(&dir);
        std::fs::create_dir_all(&dir).expect("test dir");
        guard
    }

    #[test]
    fn load_returns_default_when_missing() {
        let _guard = isolated_storage("default");
        let config = load_json_config(
            "tench-config-test",
            "config.json",
            TestConfig {
                value: "default".to_string(),
            },
        )
        .expect("load");

        assert_eq!(config.value, "default");
    }

    #[test]
    fn save_and_load_round_trips_json_config() {
        let _guard = isolated_storage("roundtrip");
        let config = TestConfig {
            value: "saved".to_string(),
        };

        save_json_config("tench-config-test", "config.json", &config).expect("save");
        let loaded = load_json_config(
            "tench-config-test",
            "config.json",
            TestConfig {
                value: "default".to_string(),
            },
        )
        .expect("load");

        assert_eq!(loaded, config);
    }
}
