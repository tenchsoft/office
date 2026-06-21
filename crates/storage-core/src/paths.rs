use std::path::PathBuf;

use crate::OfficeStorageArea;

pub fn office_app_data_dir(product_id: impl AsRef<str>) -> PathBuf {
    app_data_dir("Tench", product_id.as_ref())
}

pub fn office_storage_dir(product_id: impl AsRef<str>, area: OfficeStorageArea) -> PathBuf {
    office_app_data_dir(product_id).join(area.as_str())
}

pub fn app_config_dir(vendor: impl AsRef<str>, app_name: impl AsRef<str>) -> PathBuf {
    config_root().join(vendor.as_ref()).join(app_name.as_ref())
}

pub fn app_config_file(
    vendor: impl AsRef<str>,
    app_name: impl AsRef<str>,
    file_name: impl AsRef<str>,
) -> PathBuf {
    app_config_dir(vendor, app_name).join(file_name.as_ref())
}

pub fn app_data_dir(vendor: impl AsRef<str>, app_name: impl AsRef<str>) -> PathBuf {
    data_root().join(vendor.as_ref()).join(app_name.as_ref())
}

pub fn app_data_file(
    vendor: impl AsRef<str>,
    app_name: impl AsRef<str>,
    file_name: impl AsRef<str>,
) -> PathBuf {
    app_data_dir(vendor, app_name).join(file_name.as_ref())
}

fn config_root() -> PathBuf {
    if cfg!(windows) {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| fallback_home().join("AppData").join("Roaming"))
    } else if cfg!(target_os = "macos") {
        fallback_home().join("Library").join("Application Support")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| fallback_home().join(".config"))
    }
}

fn data_root() -> PathBuf {
    if cfg!(windows) {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| fallback_home().join("AppData").join("Roaming"))
    } else if cfg!(target_os = "macos") {
        fallback_home().join("Library").join("Application Support")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| fallback_home().join(".local").join("share"))
    }
}

fn fallback_home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
