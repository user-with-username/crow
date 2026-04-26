use anyhow::{Context, Result};
use std::path::Path;

pub fn change_directory(path: &Path) -> Result<()> {
    std::env::set_current_dir(path)
        .context(format!("failed to change directory to {}", path.display()))
}

pub fn normalize_path(path: &str) -> String {
    let mut result = path.to_string();

    if cfg!(windows) {
        if result.starts_with(r"\\?\") {
            result = result[4..].to_string();
        }
        if result.starts_with(r"UNC\") {
            result = format!(r"\\{}", &result[4..]);
        }
    }

    if cfg!(windows) {
        result.replace("/", "\\")
    } else {
        result.replace("\\", "/")
    }
}

pub fn fix_msvc_path(path: &str, default_extension: Option<&str>) -> String {
    let mut fixed = normalize_path(path);

    if let Some(ext) = default_extension {
        if !fixed.ends_with(ext) && !fixed.ends_with(&ext.to_uppercase()) {
            if fixed.contains('.') {
                if let Some(pos) = fixed.rfind('.') {
                    fixed = format!("{}{}", &fixed[..pos], ext);
                }
            } else {
                fixed.push_str(ext);
            }
        }
    }

    fixed
}
