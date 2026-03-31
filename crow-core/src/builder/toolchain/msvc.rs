use crate::builder::kinds::archiver_kind::ArchiverKind;
use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;
use crate::builder::toolchain::Toolchain;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MsvcToolchain {
    compiler_path: String,
    linker_path: String,
    archiver_path: String,
    archiver_kind: ArchiverKind,
    system_includes: Vec<PathBuf>,
    system_libraries: Vec<PathBuf>,
}

impl MsvcToolchain {
    pub fn identify_compiler(compiler_exe: &str) -> Option<CompilerKind> {
        if !cfg!(target_os = "windows") {
            return None;
        }

        let output = Command::new(compiler_exe)
            .args(&["/nologo", "/?"])
            .output()
            .ok()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);

        if combined.contains("Microsoft (R) C/C++ Optimizing Compiler") {
            return Some(CompilerKind::Msvc);
        }

        let version_check = Command::new(compiler_exe)
            .arg("--version")
            .output()
            .ok()?;
        let version_out = String::from_utf8_lossy(&version_check.stdout);
        if version_out.contains("clang") && version_out.contains("clang-cl") {
            return Some(CompilerKind::ClangCl);
        }

        None
    }

    pub fn identify_linker(linker_exe: &str) -> bool {
        if !cfg!(target_os = "windows") {
            return false;
        }

        let mut supports_msvc = false;
        let mut supports_gcc = false;

        if let Ok(output) = Command::new(linker_exe).arg("/?").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);
            if combined.contains("Microsoft (R) Incremental Linker") {
                supports_msvc = true;
            } else if output.status.success() {
                supports_msvc = true;
            }
        }

        if let Ok(output) = Command::new(linker_exe).arg("--version").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);
            if combined.contains("GNU ld")
                || combined.contains("GNU gold")
                || combined.contains("LLD")
                || combined.contains("ld64")
            {
                supports_gcc = true;
            } else if output.status.success() {
                supports_gcc = true;
            }
        }

        if !supports_gcc {
            if let Ok(output) = Command::new(linker_exe).arg("-v").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}{}", stdout, stderr);
                if combined.contains("GNU ld") || combined.contains("GNU gold") || combined.contains("LLD") {
                    supports_gcc = true;
                } else if output.status.success() {
                    supports_gcc = true;
                }
            }
        }

        supports_msvc || (supports_msvc && supports_gcc)
    }

    pub fn detect(
        preferred_compiler: Option<String>,
        preferred_linker: Option<String>,
        preferred_archiver: Option<String>,
        preferred_archiver_kind: Option<ArchiverKind>,
    ) -> Result<Self> {
        if !cfg!(target_os = "windows") {
            anyhow::bail!("MSVC toolchain is only available on Windows");
        }

        match preferred_compiler {
            Some(path) => Self::from_compiler_path(&path, preferred_linker, preferred_archiver, preferred_archiver_kind),
            None => Self::from_vs_installation_with_linker_and_archiver(preferred_linker, preferred_archiver, preferred_archiver_kind),
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

        Err(anyhow!("Could not find '{}' in PATH or current directory", spec))
    }

    fn determine_target_arch(compiler_path: &Path) -> String {
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

    fn from_compiler_path(
        compiler_spec: &str,
        preferred_linker: Option<String>,
        preferred_archiver: Option<String>,
        preferred_archiver_kind: Option<ArchiverKind>,
    ) -> Result<Self> {
        let compiler_path = Self::resolve_to_full_path(compiler_spec)?;

        let is_clang_cl = compiler_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase().contains("clang-cl"))
            .unwrap_or(false);

        if is_clang_cl && !compiler_path.to_string_lossy().contains("VC\\Tools\\MSVC") {
            return Self::from_vs_installation_with_linker_and_archiver(preferred_linker, preferred_archiver, preferred_archiver_kind);
        }

        let target_arch = Self::determine_target_arch(&compiler_path);

        let linker_path = if let Some(linker) = preferred_linker {
            Self::resolve_to_full_path(&linker)?
        } else {
            compiler_path
                .parent()
                .map(|p| p.join("link.exe"))
                .filter(|p| p.is_file())
                .ok_or_else(|| anyhow!("Could not find link.exe next to compiler"))?
        };

        let (archiver_path, archiver_kind) = if let Some(path) = preferred_archiver {
            let kind = preferred_archiver_kind.unwrap_or(ArchiverKind::Lib);
            (Self::resolve_to_full_path(&path)?, kind)
        } else {
            let default_lib = compiler_path
                .parent()
                .map(|p| p.join("lib.exe"))
                .filter(|p| p.is_file())
                .ok_or_else(|| anyhow!("Could not find lib.exe next to compiler"))?;
            (default_lib, ArchiverKind::Lib)
        };

        let tools_root = compiler_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
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
            archiver_path: archiver_path.to_string_lossy().to_string(),
            archiver_kind,
            system_includes: includes,
            system_libraries: libraries,
        })
    }

    fn from_vs_installation_with_linker_and_archiver(
        preferred_linker: Option<String>,
        preferred_archiver: Option<String>,
        preferred_archiver_kind: Option<ArchiverKind>,
    ) -> Result<Self> {
        let (tools_root, target_arch) = Self::find_vs_tools_root()?;

        let compiler_path = tools_root
            .join("bin")
            .join("Hostx64")
            .join(&target_arch)
            .join("cl.exe");

        let linker_path = if let Some(linker) = preferred_linker {
            Self::resolve_to_full_path(&linker)?
        } else {
            tools_root
                .join("bin")
                .join("Hostx64")
                .join(&target_arch)
                .join("link.exe")
        };

        let (archiver_path, archiver_kind) = if let Some(path) = preferred_archiver {
            let kind = preferred_archiver_kind.unwrap_or(ArchiverKind::Lib);
            (Self::resolve_to_full_path(&path)?, kind)
        } else {
            let default_lib = tools_root
                .join("bin")
                .join("Hostx64")
                .join(&target_arch)
                .join("lib.exe");
            (default_lib, ArchiverKind::Lib)
        };

        if !compiler_path.exists() {
            return Err(anyhow!("Compiler not found at {}", compiler_path.display()));
        }
        if !linker_path.exists() {
            return Err(anyhow!("Linker not found at {}", linker_path.display()));
        }
        if !archiver_path.exists() {
            return Err(anyhow!("Archiver not found at {}", archiver_path.display()));
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
            archiver_path: archiver_path.to_string_lossy().to_string(),
            archiver_kind,
            system_includes: includes,
            system_libraries: libraries,
        })
    }

    fn find_vs_tools_root() -> Result<(PathBuf, String)> {
        let vswhere = Self::find_vswhere()
            .ok_or_else(|| anyhow!("vswhere.exe not found. Is Visual Studio / Build Tools installed?"))?;

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
                if latest_version.as_ref().map_or(true, |v: &String| &version > v) {
                    latest_version = Some(version);
                }
            }
        }

        let version_dir = latest_version.ok_or_else(|| anyhow!("No MSVC tools version found"))?;
        let tools_root = vc_tools_path.join(&version_dir);
        let target_arch = "x64".to_string();

        Ok((tools_root, target_arch))
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

    fn archiver_path(&self) -> &str {
        &self.archiver_path
    }

    fn archiver_kind(&self) -> ArchiverKind {
        self.archiver_kind
    }
}