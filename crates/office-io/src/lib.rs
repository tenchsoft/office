pub mod backup;
pub mod config_io;
pub mod diagnostic;
pub mod docs;
pub mod error;
pub mod file_util;
pub mod office_file;
pub mod recent_files;
pub mod recovery;
pub mod sheets;
pub mod slides;
pub mod watcher;
pub mod xml_util;
pub mod zip_util;

#[cfg(test)]
mod test_util {
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    /// Global lock for tests that mutate process-wide data-home environment.
    /// All test modules must use this to avoid race conditions.
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn set_test_data_home(path: &Path) {
        std::env::set_var("XDG_DATA_HOME", path);
        std::env::set_var("APPDATA", path);
    }
}
