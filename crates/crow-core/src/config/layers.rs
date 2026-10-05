use anyhowed::{Context, Result};
use std::path::{Component, Path, PathBuf};

const CONFIG_DIR: &str = ".crow";
const CONFIG_FILE: &str = "config.toml";

pub(crate) fn read_layers(dir: &Path) -> Result<Vec<(String, String)>> {
    config_files(dir)
        .into_iter()
        .map(|path| {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read config at {}", path.display()))?;
            Ok((path.display().to_string(), content))
        })
        .collect()
}

fn config_files(dir: &Path) -> Vec<PathBuf> {
    let dir = absolute(dir);

    let mut files: Vec<PathBuf> = dir
        .ancestors()
        .map(|ancestor| ancestor.join(CONFIG_DIR).join(CONFIG_FILE))
        .filter(|path| path.is_file())
        .collect();
    files.reverse();
    files
}

fn absolute(path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push(component.as_os_str());
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
