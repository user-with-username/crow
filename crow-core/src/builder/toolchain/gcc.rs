use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;
use crate::builder::toolchain::Toolchain;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct GccToolchain {
    compiler_kind: CompilerKind,
    compiler_path: String,
    system_includes: Vec<PathBuf>,
    system_libraries: Vec<PathBuf>,
}

impl GccToolchain {
    pub fn detect(preferred_path: Option<String>) -> Result<Self> {
        let (compiler_exe, compiler_kind) = match preferred_path {
            Some(path) => {
                let kind = Self::detect_compiler_kind(&path)?;
                (path, kind)
            }
            None => {
                #[cfg(target_os = "macos")]
                let default = "clang++";
                #[cfg(not(target_os = "macos"))]
                let default = "g++";

                let compiler_exe = default.to_string();
                let kind = Self::detect_compiler_kind(&compiler_exe)?;
                (compiler_exe, kind)
            }
        };

        let system_includes = Self::extract_system_includes(&compiler_exe)?;
        let system_libraries = Self::extract_system_library_dirs(&compiler_exe)?;

        Ok(GccToolchain {
            compiler_kind,
            compiler_path: compiler_exe,
            system_includes,
            system_libraries,
        })
    }

    fn detect_compiler_kind(compiler_exe: &str) -> Result<CompilerKind> {
        let output = Command::new(compiler_exe)
            .args(&["-dM", "-E", "-"])
            .stdin(std::process::Stdio::null())
            .output()
            .context("Failed to run compiler for macro detection")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let is_clang = stdout.contains("__clang__");
        let is_gnu = stdout.contains("__GNUC__");

        let kind = if is_clang {
            if compiler_exe.contains("++") {
                CompilerKind::ClangPP
            } else {
                CompilerKind::Clang
            }
        } else if is_gnu {
            if compiler_exe.contains("++") {
                CompilerKind::Gpp
            } else {
                CompilerKind::Gcc
            }
        } else {
            CompilerKind::Gpp
        };
        Ok(kind)
    }

    fn extract_system_includes(compiler_exe: &str) -> Result<Vec<PathBuf>> {
        let output = Command::new(compiler_exe)
            .args(&["-E", "-x", "c++", "-", "-v"])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output()
            .context("Failed to run compiler for include path extraction")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut includes = Vec::new();
        let mut in_include_block = false;

        for line in stderr.lines() {
            if line.contains("#include <...> search starts here:") {
                in_include_block = true;
                continue;
            }
            if in_include_block {
                if line.contains("End of search list.") {
                    break;
                }
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    includes.push(PathBuf::from(trimmed));
                }
            }
        }
        Ok(includes)
    }

    fn extract_system_library_dirs(compiler_exe: &str) -> Result<Vec<PathBuf>> {
        let mut libraries = Vec::new();

        if let Ok(output) = Command::new(compiler_exe)
            .arg("-print-search-dirs")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("libraries: =") {
                    let paths = line.trim_start_matches("libraries: =");
                    for path in paths.split(':') {
                        let path = path.trim();
                        if !path.is_empty() {
                            libraries.push(PathBuf::from(path));
                        }
                    }
                    break;
                }
            }
        }

        if libraries.is_empty() {
            if let Ok(output) = Command::new(compiler_exe)
                .args(&["-Wl,--verbose", "-shared"])
                .stderr(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .output()
            {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let combined = format!("{}\n{}", stderr, stdout);

                for line in combined.lines() {
                    if line.contains("SEARCH_DIR") {
                        if let Some(start) = line.find('"') {
                            if let Some(end) = line.rfind('"') {
                                if start < end {
                                    let path = &line[start + 1..end];
                                    let path = path.strip_prefix('=').unwrap_or(path);
                                    libraries.push(PathBuf::from(path));
                                }
                            }
                        }
                    }

                    #[cfg(target_os = "macos")]
                    {
                        if line.contains("Library search paths:")
                            || line.starts_with(' ') && line.contains('/')
                        {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() && trimmed.starts_with('/') {
                                libraries.push(PathBuf::from(trimmed));
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        if libraries.is_empty() {
            if let Ok(output) = Command::new("ld").arg("--verbose").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("SEARCH_DIR") {
                        if let Some(start) = line.find('"') {
                            if let Some(end) = line.rfind('"') {
                                if start < end {
                                    let path = &line[start + 1..end];
                                    let path = path.strip_prefix('=').unwrap_or(path);
                                    libraries.push(PathBuf::from(path));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new(compiler_exe)
            .args(&[
                "-print-multiarch",
                "-print-sysroot",
                "-print-file-name=libc.so",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed != "libc.so" {
                    if trimmed.contains('/') {
                        if let Some(parent) = PathBuf::from(trimmed).parent() {
                            if !libraries.contains(&parent.to_path_buf()) {
                                libraries.push(parent.to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new(compiler_exe).args(&["-dumpspecs"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("lib") && line.contains('/') {
                    if let Some(start) = line.find('{') {
                        if let Some(end) = line.find('}') {
                            if start < end {
                                let content = &line[start + 1..end];
                                for word in content.split_whitespace() {
                                    if word.contains('/') && !word.contains('*') {
                                        libraries.push(PathBuf::from(word));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut seen = std::collections::HashSet::new();
        libraries.retain(|path| {
            let path_str = path.to_string_lossy().to_string();
            if seen.contains(&path_str) || !path.exists() {
                false
            } else {
                seen.insert(path_str);
                true
            }
        });

        Ok(libraries)
    }
}

impl Toolchain for GccToolchain {
    fn compiler_kind(&self) -> CompilerKind {
        self.compiler_kind
    }

    fn linker_kind(&self) -> LinkerKind {
        match self.compiler_kind {
            CompilerKind::Gcc | CompilerKind::Gpp => LinkerKind::Ld,
            CompilerKind::Clang | CompilerKind::ClangPP => LinkerKind::Lld,
            _ => LinkerKind::Unknown,
        }
    }

    fn compiler_path(&self) -> &str {
        &self.compiler_path
    }

    fn linker_path(&self) -> &str {
        &self.compiler_path
    }

    fn system_include_dirs(&self) -> Vec<PathBuf> {
        self.system_includes.clone()
    }

    fn system_library_dirs(&self) -> Vec<PathBuf> {
        self.system_libraries.clone()
    }
}
