use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

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

pub fn find_executable(bin_name: &str) -> Result<PathBuf> {
    if let Ok(path) = which::which(bin_name) {
        return Ok(path);
    }

    #[cfg(windows)]
    {
        let exe_name = format!("{}.exe", bin_name);
        let program_files =
            std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 = std::env::var("ProgramFiles(x86)")
            .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());

        let common_install_dirs = vec![
            program_files,
            program_files_x86,
            "C:\\Program Files".to_string(),
            "C:\\Program Files (x86)".to_string(),
        ];

        for base_dir in common_install_dirs {
            for dir_name in &[
                bin_name,
                &bin_name.to_uppercase(),
                &format!("{}-*", bin_name),
            ] {
                let pattern = PathBuf::from(&base_dir)
                    .join(dir_name)
                    .join("bin")
                    .join(&exe_name);
                if pattern.exists() {
                    return Ok(pattern);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let possible_paths = vec![
            format!("/usr/local/bin/{}", bin_name),
            format!("/opt/homebrew/bin/{}", bin_name),
            format!("/opt/local/bin/{}", bin_name),
        ];
        for path in possible_paths {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let possible_paths = vec![
            format!("/usr/bin/{}", bin_name),
            format!("/usr/local/bin/{}", bin_name),
        ];
        for path in possible_paths {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }
    }

    anyhow::bail!(
        "{} not found in PATH or common installation directories",
        bin_name
    )
}
