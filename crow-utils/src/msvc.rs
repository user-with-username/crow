use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_msvc_system_paths(target_arch: &str) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let vs_root = find_vs_root()?;
    let vc_tools_root = vs_root.join("VC").join("Tools").join("MSVC");
    let latest_version = find_latest_msvc_version(&vc_tools_root)?;
    let tools_root = vc_tools_root.join(&latest_version);

    let mut includes = Vec::new();
    let mut libraries = Vec::new();

    let vc_include = tools_root.join("include");
    if vc_include.exists() {
        includes.push(vc_include);
    }

    let vc_lib = tools_root.join("lib").join(target_arch);
    if vc_lib.exists() {
        libraries.push(vc_lib);
    }

    add_windows_sdk_paths(&mut includes, &mut libraries, target_arch)?;

    Ok((includes, libraries))
}

pub fn determine_target_arch(compiler_path: &Path) -> String {
    if let Ok(target) = std::env::var("TARGET") {
        if target.contains("64") {
            return "x64".to_string();
        } else if target.contains("86") {
            return "x86".to_string();
        }
    }

    let parent = compiler_path.parent().and_then(|p| p.file_name());
    let grandparent = compiler_path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name());

    match (grandparent, parent) {
        (Some(gp), Some(p)) => {
            let gp_str = gp.to_string_lossy().to_lowercase();
            let p_str = p.to_string_lossy().to_lowercase();

            if gp_str.contains("hostx64") || p_str == "x64" || p_str == "amd64" {
                "x64".to_string()
            } else if gp_str.contains("hostx86") || p_str == "x86" || p_str == "win32" {
                "x86".to_string()
            } else {
                "x64".to_string()
            }
        }
        _ => "x64".to_string(),
    }
}

fn find_vs_root() -> Result<PathBuf> {
    if let Some(vswhere) = find_vswhere() {
        let output = Command::new(&vswhere)
            .args(&[
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
            ])
            .output()
            .context("Failed to run vswhere")?;

        let stdout = String::from_utf8(output.stdout)?;
        let vs_path = stdout.trim();
        if !vs_path.is_empty() && Path::new(vs_path).exists() {
            return Ok(PathBuf::from(vs_path));
        }
    }

    let candidates = [
        r"C:\Program Files\Microsoft Visual Studio\2022\Community",
        r"C:\Program Files\Microsoft Visual Studio\2022\Professional",
        r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional",
        r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise",
    ];
    for candidate in candidates.iter() {
        if Path::new(candidate).exists() {
            return Ok(PathBuf::from(candidate));
        }
    }

    Err(anyhow!("Could not locate Visual Studio installation"))
}

fn find_latest_msvc_version(vc_tools_root: &Path) -> Result<String> {
    let entries = std::fs::read_dir(vc_tools_root).context("Failed to read VC tools directory")?;
    let mut versions = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map_or(false, |ft| ft.is_dir()) {
            if let Some(version) = entry.file_name().to_str() {
                versions.push(version.to_string());
            }
        }
    }
    versions.sort_by(|a, b| {
        fn version_parts(v: &str) -> Vec<u32> {
            v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect()
        }
        let a_parts = version_parts(a);
        let b_parts = version_parts(b);
        a_parts.cmp(&b_parts)
    });
    versions
        .last()
        .cloned()
        .ok_or_else(|| anyhow!("No MSVC toolchain found"))
}

fn add_windows_sdk_paths(
    includes: &mut Vec<PathBuf>,
    libraries: &mut Vec<PathBuf>,
    target_arch: &str,
) -> Result<()> {
    let sdk_root = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10");
    if !sdk_root.exists() {
        return Ok(());
    }

    let include_root = sdk_root.join("Include");
    let lib_root = sdk_root.join("Lib");

    if let Ok(entries) = std::fs::read_dir(&include_root) {
        let mut sdk_versions = Vec::new();
        for entry in entries.flatten() {
            if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                sdk_versions.push(entry.file_name().to_string_lossy().to_string());
            }
        }

        sdk_versions.sort_by(|a, b| {
            fn version_parts(v: &str) -> Vec<u32> {
                v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect()
            }
            let a_parts = version_parts(a);
            let b_parts = version_parts(b);
            a_parts.cmp(&b_parts)
        });

        if let Some(latest) = sdk_versions.last() {
            let sdk_include = include_root.join(latest);

            let inc_paths = ["um", "shared", "winrt", "ucrt"];

            for sub in inc_paths {
                let p = sdk_include.join(sub);
                if p.exists() {
                    includes.push(p);
                }
            }

            let sdk_lib = lib_root.join(latest);

            let lib_sub_dirs = [
                format!("ucrt/{}", target_arch),
                format!("um/{}", target_arch),
            ];

            for sub in lib_sub_dirs {
                let p = sdk_lib.join(sub);
                if p.exists() {
                    libraries.push(p);
                }
            }
        }
    }

    Ok(())
}

fn find_vswhere() -> Option<PathBuf> {
    let candidates = [
        r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
        r"C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe",
    ];
    for path in candidates.iter() {
        if Path::new(path).exists() {
            return Some(PathBuf::from(path));
        }
    }
    None
}