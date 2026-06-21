use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

use tench_fs_core::{FileWatchEventKind, OfficeFileWatchEvent};

pub struct OfficeFileWatcher {
    product_id: String,
    watched: HashMap<String, SystemTime>,
}

impl OfficeFileWatcher {
    pub fn new(product_id: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            watched: HashMap::new(),
        }
    }

    pub fn watch(&mut self, path: impl AsRef<str>) {
        let path = path.as_ref();
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.watched.insert(path.to_string(), modified);
            }
        }
    }

    pub fn unwatch(&mut self, path: &str) {
        self.watched.remove(path);
    }

    pub fn check_changes(&self) -> Vec<OfficeFileWatchEvent> {
        let mut events = Vec::new();

        for (path, last_modified) in &self.watched {
            match fs::metadata(path) {
                Ok(metadata) => {
                    if let Ok(current_modified) = metadata.modified() {
                        if current_modified != *last_modified {
                            events.push(self.event(path, FileWatchEventKind::ExternalModified));
                        }
                    }
                }
                Err(error) => {
                    let kind = if error.kind() == std::io::ErrorKind::PermissionDenied {
                        FileWatchEventKind::PermissionDenied
                    } else if error.kind() == std::io::ErrorKind::NotFound {
                        FileWatchEventKind::Deleted
                    } else {
                        FileWatchEventKind::ExternalModified
                    };
                    events.push(self.event(path, kind));
                }
            }
        }

        events
    }

    pub fn mark_self_saved(&mut self, path: &str) {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.watched.insert(path.to_string(), modified);
            }
        }
    }

    pub fn self_saved_event(&self, path: &str) -> OfficeFileWatchEvent {
        self.event(path, FileWatchEventKind::SelfSaved)
    }

    pub fn watched_count(&self) -> usize {
        self.watched.len()
    }

    fn event(&self, path: &str, kind: FileWatchEventKind) -> OfficeFileWatchEvent {
        OfficeFileWatchEvent {
            product_id: self.product_id.clone(),
            path: path.to_string(),
            kind,
            previous_path: None,
            marker: None,
            observed_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tench_office_watcher_{name}_{}_{}",
            std::process::id(),
            crate::file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).expect("test dir");
        dir
    }

    #[test]
    fn watch_registers_existing_file() {
        let dir = unique_temp_dir("register");
        let file = dir.join("test.txt");
        std::fs::write(&file, "hello").expect("write");

        let mut watcher = OfficeFileWatcher::new("tench-test");
        watcher.watch(file.to_string_lossy());

        assert_eq!(watcher.watched_count(), 1);
        assert!(watcher.check_changes().is_empty());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_deletion() {
        let dir = unique_temp_dir("delete");
        let file = dir.join("test.txt");
        std::fs::write(&file, "hello").expect("write");

        let mut watcher = OfficeFileWatcher::new("tench-test");
        watcher.watch(file.to_string_lossy());
        std::fs::remove_file(&file).expect("remove");

        let events = watcher.check_changes();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].product_id, "tench-test");
        assert_eq!(events[0].kind, FileWatchEventKind::Deleted);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn mark_self_saved_updates_baseline() {
        let dir = unique_temp_dir("self_saved");
        let file = dir.join("test.txt");
        std::fs::write(&file, "hello").expect("write");

        let mut watcher = OfficeFileWatcher::new("tench-test");
        watcher.watch(file.to_string_lossy());
        std::fs::write(&file, "self-saved").expect("write modified");
        watcher.mark_self_saved(&file.to_string_lossy());

        assert!(watcher.check_changes().is_empty());
        assert_eq!(
            watcher.self_saved_event(&file.to_string_lossy()).kind,
            FileWatchEventKind::SelfSaved
        );

        let _ = std::fs::remove_dir_all(dir);
    }
}
