use crate::builder::kinds::archiver_kind::ArchiverKind;
use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;
use crate::builder::toolchain::Toolchain;
use crow_utils::msvc;
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

        let version_check = Command::new(compiler_exe).arg("--version").output().ok()?;
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

        supports_msvc
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
            Some(path) => Self::from_compiler_path(
                &path,
                preferred_linker,
                preferred_archiver,
                preferred_archiver_kind,
            ),
            None => Self::from_vs_installation(
                preferred_linker,
                preferred_archiver,
                preferred_archiver_kind,
            ),
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

        Err(anyhow!(
            "Could not find '{}' in PATH or current directory",
            spec
        ))
    }

    fn from_compiler_path(
        compiler_spec: &str,
        preferred_linker: Option<String>,
        preferred_archiver: Option<String>,
        preferred_archiver_kind: Option<ArchiverKind>,
    ) -> Result<Self> {
        let compiler_path = Self::resolve_to_full_path(compiler_spec)?;
        let target_arch = msvc::determine_target_arch(&compiler_path);

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
            let kind = if let Some(k) = preferred_archiver_kind {
                k
            } else {
                Self::detect_archiver_kind(&path)?
            };
            (Self::resolve_to_full_path(&path)?, kind)
        } else {
            let default_lib = compiler_path
                .parent()
                .map(|p| p.join("lib.exe"))
                .filter(|p| p.is_file())
                .ok_or_else(|| anyhow!("Could not find lib.exe next to compiler"))?;
            (default_lib, ArchiverKind::Lib)
        };

        let (system_includes, system_libraries) = msvc::get_msvc_system_paths(&target_arch)?;

        Ok(MsvcToolchain {
            compiler_path: compiler_path.to_string_lossy().to_string(),
            linker_path: linker_path.to_string_lossy().to_string(),
            archiver_path: archiver_path.to_string_lossy().to_string(),
            archiver_kind,
            system_includes,
            system_libraries,
        })
    }

    fn from_vs_installation(
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
            let kind = if let Some(k) = preferred_archiver_kind {
                k
            } else {
                Self::detect_archiver_kind(&path)?
            };
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

        let (system_includes, system_libraries) = msvc::get_msvc_system_paths(&target_arch)?;

        Ok(MsvcToolchain {
            compiler_path: compiler_path.to_string_lossy().to_string(),
            linker_path: linker_path.to_string_lossy().to_string(),
            archiver_path: archiver_path.to_string_lossy().to_string(),
            archiver_kind,
            system_includes,
            system_libraries,
        })
    }

    fn find_vs_tools_root() -> Result<(PathBuf, String)> {
        let vswhere = Self::find_vswhere().ok_or_else(|| {
            anyhow!("vswhere.exe not found. Is Visual Studio / Build Tools installed?")
        })?;

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

        let vc_versions =
            std::fs::read_dir(&vc_tools_path).context("Failed to read VC tools directory")?;

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

        Ok((tools_root, target_arch))
    }

    fn detect_archiver_kind(archiver_exe: &str) -> Result<ArchiverKind> {
        if let Ok(output) = Command::new(archiver_exe).arg("/?").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);
            if combined.contains("Microsoft (R) Library Manager") {
                return Ok(ArchiverKind::Lib);
            }
        }

        if let Ok(output) = Command::new(archiver_exe).arg("--version").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr).to_lowercase();
            if combined.contains("llvm") {
                return Ok(ArchiverKind::LlvmAr);
            } else if combined.contains("gnu ar") || combined.contains("gcc-ar") {
                return Ok(ArchiverKind::Ar);
            }
        }

        Ok(ArchiverKind::Lib)
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