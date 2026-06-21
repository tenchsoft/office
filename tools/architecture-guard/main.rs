//! Architecture guard — verifies file line counts and Tauri command counts stay within budget.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_LINE_BUDGET: usize = 300;
const IGNORED_DIRS: &[&str] = &[".git", "dist", "gen", "node_modules", "target"];
const SOURCE_EXTENSIONS: &[&str] = &["rs"];
const LINE_BUDGET_BASELINE: &str = include_str!("line_budget_baseline.txt");
const COMMAND_BUDGET_BASELINE: &str = "\
apps/docs/src-tauri/src/commands.rs 62
apps/kodocs/src-tauri/src/commands.rs 62
";

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("cannot find repo root");

    let strict = std::env::args().any(|a| a == "--strict")
        || std::env::var("TENCH_ARCH_STRICT").as_deref() == Ok("1");

    let mut failures: Vec<String> = Vec::new();
    let mut checked_files = 0;
    let mut tauri_commands = 0;
    let line_baseline = parse_line_budget_baseline(LINE_BUDGET_BASELINE);
    let command_baseline = parse_line_budget_baseline(COMMAND_BUDGET_BASELINE);

    let files = walk_files(root, "apps", SOURCE_EXTENSIONS);
    let crate_files = walk_files(root, "crates", SOURCE_EXTENSIONS);
    let all_files: Vec<PathBuf> = files.into_iter().chain(crate_files).collect();

    for absolute_path in &all_files {
        let relative = path_diff(root, absolute_path);
        let extension = absolute_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let source = fs::read_to_string(absolute_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", relative));
        let lines = source.lines().count();
        checked_files += 1;

        // Line budget
        let line_budget = if relative.contains("src-tauri/src/lib.rs") {
            160
        } else {
            DEFAULT_LINE_BUDGET
        };

        let baseline_budget = line_baseline
            .get(relative.as_str())
            .copied()
            .unwrap_or(line_budget);
        if strict && lines > line_budget && lines > baseline_budget {
            failures.push(format!(
                "{relative} has {lines} lines, budget is {line_budget}, baseline is {baseline_budget}"
            ));
        }

        // Tauri command count
        if relative.contains("src-tauri/") && extension == "rs" {
            let count = count_tauri_commands(&source);
            if count > 0 {
                tauri_commands += count;
                let cmd_budget = command_budget(&relative);
                let baseline_budget = command_baseline
                    .get(relative.as_str())
                    .copied()
                    .unwrap_or(cmd_budget);
                if count > cmd_budget && count > baseline_budget {
                    failures.push(format!(
                        "{relative} has {count} Tauri command definitions, budget is {cmd_budget}, baseline is {baseline_budget}"
                    ));
                }
            }
        }
    }

    if failures.is_empty() {
        println!("architecture guard: ok ({checked_files} files, {tauri_commands} Tauri commands)");
    } else {
        for failure in &failures {
            eprintln!("architecture guard failed: {failure}");
        }
        std::process::exit(1);
    }
}

fn walk_files(root: &Path, start_dir: &str, extensions: &[&str]) -> Vec<PathBuf> {
    let ext_set: HashSet<&str> = extensions.iter().copied().collect();
    let ignored: HashSet<&str> = IGNORED_DIRS.iter().copied().collect();
    let mut files = Vec::new();
    let mut stack = vec![root.join(start_dir)];

    while let Some(dir) = stack.pop() {
        if !dir.exists() {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if ignored.contains(name_str.as_ref()) {
                continue;
            }
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext_set.contains(ext) {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    files
}

fn path_diff(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn count_tauri_commands(source: &str) -> usize {
    let mut count = 0;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[tauri::command") || trimmed.starts_with("#[command") {
            count += 1;
        }
    }
    count
}

fn command_budget(relative: &str) -> usize {
    if relative.contains("src-tauri/src/lib.rs") {
        0
    } else if relative.contains("src-tauri/src/commands.rs")
        || relative.contains("src-tauri/src/commands/")
    {
        50
    } else {
        0
    }
}

fn parse_line_budget_baseline(content: &str) -> HashMap<String, usize> {
    let mut baseline = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((path, lines)) = trimmed.rsplit_once(' ') else {
            continue;
        };
        if let Ok(lines) = lines.parse::<usize>() {
            baseline.insert(path.to_string(), lines);
        }
    }
    baseline
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_files_matches_rust_extension_without_dot() {
        let root = std::env::temp_dir().join(format!(
            "tench_architecture_guard_ext_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let app_dir = root.join("apps/demo/src");
        fs::create_dir_all(&app_dir).expect("test dir");
        fs::write(app_dir.join("lib.rs"), "fn main() {}\n").expect("test source");
        fs::write(app_dir.join("notes.md"), "# ignored\n").expect("test markdown");

        let files = walk_files(&root, "apps", SOURCE_EXTENSIONS);

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("lib.rs"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn line_budget_baseline_uses_last_space_as_separator() {
        let baseline = parse_line_budget_baseline("apps/demo file/src/lib.rs 301\n");

        assert_eq!(
            baseline.get("apps/demo file/src/lib.rs").copied(),
            Some(301)
        );
    }
}
