use std::path::PathBuf;

pub fn default_model_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(value) = std::env::var("TENCH_MODEL_DIR") {
        dirs.extend(
            value
                .split(':')
                .filter(|part| !part.is_empty())
                .map(PathBuf::from),
        );
    }

    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".tench").join("models"));
    }

    if let Ok(current_dir) = std::env::current_dir() {
        dirs.push(current_dir.join("models"));
    }

    dirs
}
