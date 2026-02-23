use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;
use crate::builder::toolchain::Toolchain;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MsvcToolchain {
    compiler_path: String,
    linker_path: String,
    system_includes: Vec<PathBuf>,
    system_libraries: Vec<PathBuf>,
}

impl MsvcToolchain {
    pub fn detect(preferred_path: Option<String>) -> Result<Self> {
        match preferred_path {
            Some(path) => Self::from_compiler_path(&path),
            None => Self::from_vs_installation(),
        }
    }

    fn resolve_to_full_path(spec: &str) -> Result<PathBuf> {
        let pb = PathBuf::from(spec);
        if pb.is_file() {
            return Ok(pb);
        }

        if let Ok(path_env) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_env) {
                let candidate = dir.join(spec);
                if candidate.is_file() {
                    return Ok(candidate);
                }

                if !spec.to_ascii_lowercase().ends_with(".exe") {
                    let candidate_exe = dir.join(format!("{}.exe", spec));
                    if candidate_exe.is_file() {
                        return Ok(candidate_exe);
                    }
                }
            }
        }

        if let Ok(cwd) = std::env::current_dir() {
            let candidate = cwd.join(spec);
            if candidate.is_file() {
                return Ok(candidate);
            }
            if !spec.to_ascii_lowercase().ends_with(".exe") {
                let candidate_exe = cwd.join(format!("{}.exe", spec));
                if candidate_exe.is_file() {
                    return Ok(candidate_exe);
                }
            }
        }

        Err(anyhow!("Could not find compiler '{}' in PATH or current directory", spec))
    }

    fn determine_target_arch(compiler_path: &Path) -> String {
        let parent = compiler_path.parent().and_then(|p| p.file_name());
        let grandparent = compiler_path.parent().and_then(|p| p.parent()).and_then(|p| p.file_name());

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

    fn from_compiler_path(compiler_spec: &str) -> Result<Self> {
        let compiler_path = Self::resolve_to_full_path(compiler_spec)?;
        let target_arch = Self::determine_target_arch(&compiler_path);

        let linker_path = compiler_path
            .parent()
            .map(|p| p.join("link.exe"))
            .filter(|p| p.is_file())
            .ok_or_else(|| anyhow!("Could not find link.exe next to compiler"))?;

        let tools_root = compiler_path
            .parent()           // x64 / x86
            .and_then(|p| p.parent())  // Hostx64 / Hostx86
            .and_then(|p| p.parent())  // bin
            .and_then(|p| p.parent())  // version
            .ok_or_else(|| anyhow!("Could not derive tools root (expected .../bin/Host(x64|x86)/(x64|x86)/cl.exe)"))?;

        let mut includes = Vec::new();
        let mut libraries = Vec::new();

        let vc_include = tools_root.join("include");
        if vc_include.exists() {
            includes.push(vc_include);
        }

        let vc_lib = tools_root.join("lib").join(&target_arch);
        if vc_lib.exists() {
            libraries.push(vc_lib);
        }

        Self::add_windows_sdk_paths(&mut includes, &mut libraries, &target_arch)?;

        Ok(MsvcToolchain {
            compiler_path: compiler_path.to_string_lossy().to_string(),
            linker_path: linker_path.to_string_lossy().to_string(),
            system_includes: includes,
            system_libraries: libraries,
        })
    }

    fn from_vs_installation() -> Result<Self> {
        let vswhere_path = Self::find_vswhere()
            .ok_or_else(|| anyhow!("vswhere.exe not found. Is Visual Studio / Build Tools installed?"))?;

        let output = Command::new(&vswhere_path)
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

        let vs_path = String::from_utf8(output.stdout)?.trim().to_string();
        if vs_path.is_empty() {
            return Err(anyhow!("No Visual Studio installation with VC tools found"));
        }

        let vc_tools_path = PathBuf::from(&vs_path)
            .join("VC")
            .join("Tools")
            .join("MSVC");

        let vc_versions = std::fs::read_dir(&vc_tools_path)
            .context("Failed to read VC tools directory")?;

        let mut latest_version = None;
        for entry in vc_versions {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let version = entry.file_name().to_string_lossy().to_string();
                if latest_version
                    .as_ref()
                    .map_or(true, |v: &String| &version > v)
                {
                    latest_version = Some(version);
                }
            }
        }

        let version_dir = latest_version.ok_or_else(|| anyhow!("No MSVC tools version found"))?;
        let tools_root = vc_tools_path.join(&version_dir);

        let target_arch = "x64".to_string();

        let compiler_path = tools_root
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("cl.exe");

        let linker_path = tools_root
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("link.exe");

        if !compiler_path.exists() {
            return Err(anyhow!("Compiler not found at {}", compiler_path.display()));
        }
        if !linker_path.exists() {
            return Err(anyhow!("Linker not found at {}", linker_path.display()));
        }

        let mut includes = Vec::new();
        let mut libraries = Vec::new();

        let vc_include = tools_root.join("include");
        if vc_include.exists() {
            includes.push(vc_include);
        }

        let vc_lib = tools_root.join("lib").join(&target_arch);
        if vc_lib.exists() {
            libraries.push(vc_lib);
        }

        Self::add_windows_sdk_paths(&mut includes, &mut libraries, &target_arch)?;

        Ok(MsvcToolchain {
            compiler_path: compiler_path.to_string_lossy().to_string(),
            linker_path: linker_path.to_string_lossy().to_string(),
            system_includes: includes,
            system_libraries: libraries,
        })
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
                    v.split('.')
                        .filter_map(|s| s.parse::<u32>().ok())
                        .collect()
                }
                let a_parts = version_parts(a);
                let b_parts = version_parts(b);
                a_parts.cmp(&b_parts)
            });

            if let Some(latest) = sdk_versions.last() {
                let sdk_include = include_root.join(latest);

                let inc_paths = [
                    "um",
                    "shared",
                    "winrt",
                    "ucrt",
                ];

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

        for path in candidates {
            if Path::new(path).exists() {
                return Some(PathBuf::from(path));
            }
        }

        None
    }
}

impl Toolchain for MsvcToolchain {
    fn compiler_kind(&self) -> CompilerKind {
        CompilerKind::Msvc
    }

    fn linker_kind(&self) -> LinkerKind {
        LinkerKind::Link
    }

    fn compiler_path(&self) -> &str {
        &self.compiler_path
    }

    fn linker_path(&self) -> &str {
        &self.linker_path
    }

    fn system_include_dirs(&self) -> Vec<PathBuf> {
        self.system_includes.clone()
    }

    fn system_library_dirs(&self) -> Vec<PathBuf> {
        self.system_libraries.clone()
    }
}