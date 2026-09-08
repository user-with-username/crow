use crate::builder::kinds::archiver_kind::ArchiverKind;
use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::builder::kinds::linker_kind::LinkerKind;
use crate::builder::toolchain::Toolchain;
use anyhowed::{Context, Result};
use crow_utils::msvc;
use std::path::PathBuf;
use std::process::Command;

pub struct GccToolchain {
    compiler_kind: CompilerKind,
    compiler_path: String,
    linker_path: String,
    archiver_path: String,
    archiver_kind: ArchiverKind,
    system_includes: Vec<PathBuf>,
    system_libraries: Vec<PathBuf>,
}

impl GccToolchain {
    pub fn identify_compiler(compiler_exe: &str) -> Option<CompilerKind> {
        Self::detect_compiler_kind(compiler_exe)
            .ok()
            .filter(|k| *k != CompilerKind::Unknown)
    }

    pub fn identify_linker(linker_exe: &str) -> bool {
        let mut supports_gcc = false;

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
            }
        }

        if !supports_gcc {
            if let Ok(output) = Command::new(linker_exe).arg("-v").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}{}", stdout, stderr);
                if combined.contains("GNU ld")
                    || combined.contains("GNU gold")
                    || combined.contains("LLD")
                {
                    supports_gcc = true;
                }
            }
        }

        supports_gcc
    }

    pub fn detect(
        preferred_compiler: Option<String>,
        preferred_linker: Option<String>,
        preferred_archiver: Option<String>,
        preferred_archiver_kind: Option<ArchiverKind>,
    ) -> Result<Self> {
        let (compiler_exe, compiler_kind) = match preferred_compiler {
            Some(path) => {
                let kind = Self::detect_compiler_kind(&path)
                    .context(format!("Failed to detect compiler kind for '{}'", path))?;
                (path, kind)
            }
            None => {
                #[cfg(target_os = "macos")]
                let default = "clang++";
                #[cfg(not(target_os = "macos"))]
                let default = "g++";

                let compiler_exe = default.to_string();
                let kind = Self::detect_compiler_kind(&compiler_exe).context(format!(
                    "Failed to detect compiler kind for default '{}'",
                    default
                ))?;
                (compiler_exe, kind)
            }
        };

        if compiler_kind == CompilerKind::Unknown {
            anyhowed::bail!(
                "Could not determine a known compiler kind for '{}'. \
                This might not be a GCC-like compiler or it's an unsupported variant.",
                compiler_exe
            );
        }

        let linker_path = if compiler_kind == CompilerKind::ClangCl && cfg!(target_os = "windows") {
            if let Some(linker) = preferred_linker {
                linker
            } else {
                if let Some(linker) = Self::find_linker_in_path("lld-link.exe")
                    .or_else(|| Self::find_linker_in_path("link.exe"))
                {
                    linker
                } else {
                    eprintln!("Warning: Could not find lld-link or link.exe in PATH; falling back to compiler as linker.");
                    compiler_exe.clone()
                }
            }
        } else {
            preferred_linker.unwrap_or_else(|| compiler_exe.clone())
        };

        let (archiver_path, archiver_kind) =
            Self::determine_archiver(&compiler_kind, preferred_archiver, preferred_archiver_kind)?;

        let (system_includes, system_libraries) = if compiler_kind == CompilerKind::ClangCl
            && cfg!(target_os = "windows")
        {
            let compiler_path = PathBuf::from(&compiler_exe);
            let target_arch = msvc::determine_target_arch(&compiler_path);
            match msvc::get_msvc_system_paths(&target_arch) {
                Ok((includes, libs)) => (includes, libs),
                Err(e) => {
                    eprintln!("Warning: Could not get MSVC paths for clang-cl: {}. Falling back to compiler extraction.", e);
                    (Vec::new(), Vec::new())
                }
            }
        } else {
            (
                Self::extract_system_includes(&compiler_exe)?,
                Self::extract_system_library_dirs(&compiler_exe)?,
            )
        };

        Ok(GccToolchain {
            compiler_kind,
            compiler_path: compiler_exe,
            linker_path,
            archiver_path,
            archiver_kind,
            system_includes,
            system_libraries,
        })
    }

    fn determine_archiver(
        compiler_kind: &CompilerKind,
        preferred_archiver: Option<String>,
        preferred_archiver_kind: Option<ArchiverKind>,
    ) -> Result<(String, ArchiverKind)> {
        if let Some(path) = preferred_archiver {
            let kind = if let Some(k) = preferred_archiver_kind {
                k
            } else {
                Self::detect_archiver_kind(&path)?
            };
            return Ok((path, kind));
        }

        let (default_path, default_kind) = match compiler_kind {
            CompilerKind::Clang | CompilerKind::ClangPP => {
                ("llvm-ar".to_string(), ArchiverKind::LlvmAr)
            }
            CompilerKind::ClangCl => ("lib.exe".to_string(), ArchiverKind::Lib),
            CompilerKind::Gcc | CompilerKind::Gpp => ("ar".to_string(), ArchiverKind::Ar),
            _ => ("ar".to_string(), ArchiverKind::Ar),
        };

        if let Some(pref_kind) = preferred_archiver_kind {
            if pref_kind != default_kind {
                anyhowed::bail!(
                    "Incompatible archiver kind: requested {:?} but default for this toolchain is {:?}",
                    pref_kind, default_kind
                );
            }
        }

        Ok((default_path, default_kind))
    }

    fn detect_archiver_kind(archiver_exe: &str) -> Result<ArchiverKind> {
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

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new(archiver_exe).arg("/?").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}{}", stdout, stderr);
                if combined.contains("Microsoft (R) Library Manager") {
                    return Ok(ArchiverKind::Lib);
                }
            }
        }

        eprintln!(
            "Warning: Could not determine archiver kind for '{}', assuming 'ar'",
            archiver_exe
        );
        Ok(ArchiverKind::Ar)
    }

    fn find_linker_in_path(exe_name: &str) -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("where").arg(exe_name).output().ok()?;
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().next().map(|s| s.trim().to_string())
            } else {
                None
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = exe_name;
            None
        }
    }

    fn detect_compiler_kind(compiler_exe: &str) -> Result<CompilerKind> {
        fn has_define(compiler: &str, args: &[&str], macro_name: &str) -> bool {
            let output = Command::new(compiler)
                .args(args)
                .stdin(std::process::Stdio::null())
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    return stdout.lines().any(|line| line.contains(macro_name));
                }
            }
            false
        }

        let version_output = Command::new(compiler_exe).arg("--version").output();
        let version_string = if let Ok(output) = &version_output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}{}", stdout, stderr).to_lowercase();
                Some(combined)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(ver) = &version_string {
            if ver.contains("clang-cl") {
                return Ok(CompilerKind::ClangCl);
            }
        }

        let is_clang = has_define(compiler_exe, &["-dM", "-E", "-"], "__clang__");
        let is_gnu = has_define(compiler_exe, &["-dM", "-E", "-"], "__GNUC__");

        if is_clang || is_gnu {
            let is_msvc = has_define(compiler_exe, &["-dM", "-E", "-"], "_MSC_VER");

            if is_clang && is_msvc {
                let supports_cpp = has_define(
                    compiler_exe,
                    &["-x", "c++", "-dM", "-E", "-"],
                    "__cplusplus",
                );
                if supports_cpp {
                    return Ok(CompilerKind::ClangPP);
                } else {
                    return Ok(CompilerKind::Clang);
                }
            }

            if is_msvc {
                return Ok(CompilerKind::Msvc);
            }

            let supports_cpp = has_define(
                compiler_exe,
                &["-x", "c++", "-dM", "-E", "-"],
                "__cplusplus",
            );

            let kind = if is_clang {
                if supports_cpp {
                    CompilerKind::ClangPP
                } else {
                    CompilerKind::Clang
                }
            } else {
                if supports_cpp {
                    CompilerKind::Gpp
                } else {
                    CompilerKind::Gcc
                }
            };
            return Ok(kind);
        }

        if let Some(ver) = version_string {
            let is_clang = ver.contains("clang");
            let is_gcc = ver.contains("gcc") || ver.contains("g++");
            let msvc_flag_supported = Command::new(compiler_exe)
                .arg("/nologo")
                .arg("/?")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            let base_kind = if is_clang && msvc_flag_supported {
                CompilerKind::ClangCl
            } else if is_clang {
                CompilerKind::Clang
            } else if is_gcc {
                if ver.contains("c++") {
                    CompilerKind::Gpp
                } else {
                    CompilerKind::Gcc
                }
            } else {
                CompilerKind::Unknown
            };

            if base_kind != CompilerKind::Unknown {
                if base_kind == CompilerKind::Clang || base_kind == CompilerKind::Gcc {
                    let supports_cpp = has_define(
                        compiler_exe,
                        &["-x", "c++", "-dM", "-E", "-"],
                        "__cplusplus",
                    );
                    return Ok(match base_kind {
                        CompilerKind::Clang if supports_cpp => CompilerKind::ClangPP,
                        CompilerKind::Gcc if supports_cpp => CompilerKind::Gpp,
                        _ => base_kind,
                    });
                }
                return Ok(base_kind);
            }
        }

        Ok(CompilerKind::Unknown)
    }

    fn extract_system_includes(compiler_exe: &str) -> Result<Vec<PathBuf>> {
        let output = Command::new(compiler_exe)
            .args(["-E", "-x", "c++", "-", "-v"])
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
                    let path = PathBuf::from(trimmed);
                    if path.exists() {
                        includes.push(path);
                    }
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
                            let pb = PathBuf::from(path);
                            if pb.exists() {
                                libraries.push(pb);
                            }
                        }
                    }
                    break;
                }
            }
        }

        if libraries.is_empty() {
            if let Ok(output) = Command::new(compiler_exe)
                .args(["-Wl,--verbose", "-shared"])
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
                                    let pb = PathBuf::from(path);
                                    if pb.exists() {
                                        libraries.push(pb);
                                    }
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
                                let pb = PathBuf::from(trimmed);
                                if pb.exists() {
                                    libraries.push(pb);
                                }
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
                                    let pb = PathBuf::from(path);
                                    if pb.exists() {
                                        libraries.push(pb);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new(compiler_exe)
            .args([
                "-print-multiarch",
                "-print-sysroot",
                "-print-file-name=libc.so",
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed != "libc.so" && trimmed.contains('/') {
                    if let Some(parent) = PathBuf::from(trimmed).parent() {
                        if parent.exists() && !libraries.contains(&parent.to_path_buf()) {
                            libraries.push(parent.to_path_buf());
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new(compiler_exe).args(["-dumpspecs"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("lib") && line.contains('/') {
                    if let Some(start) = line.find('{') {
                        if let Some(end) = line.find('}') {
                            if start < end {
                                let content = &line[start + 1..end];
                                for word in content.split_whitespace() {
                                    if word.contains('/') && !word.contains('*') {
                                        let pb = PathBuf::from(word);
                                        if pb.exists() && !libraries.contains(&pb) {
                                            libraries.push(pb);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut seen = std::collections::HashSet::new();
        let filtered: Vec<_> = libraries
            .into_iter()
            .filter(|path| {
                let path_str = path.to_string_lossy().to_string();
                if seen.contains(&path_str) {
                    false
                } else {
                    seen.insert(path_str);
                    true
                }
            })
            .collect();

        Ok(filtered)
    }
}

impl Toolchain for GccToolchain {
    fn compiler_kind(&self) -> CompilerKind {
        self.compiler_kind
    }

    fn linker_kind(&self) -> LinkerKind {
        match self.compiler_kind {
            CompilerKind::ClangCl => LinkerKind::Lld,
            CompilerKind::Gcc | CompilerKind::Gpp => LinkerKind::Ld,
            CompilerKind::Clang | CompilerKind::ClangPP => LinkerKind::Lld,
            _ => LinkerKind::Unknown,
        }
    }

    fn compiler_path(&self) -> &str {
        &self.compiler_path
    }

    fn linker_path(&self) -> &str {
        &self.linker_path
    }

    fn archiver_path(&self) -> &str {
        &self.archiver_path
    }

    fn archiver_kind(&self) -> ArchiverKind {
        self.archiver_kind
    }

    fn system_include_dirs(&self) -> Vec<PathBuf> {
        self.system_includes.clone()
    }

    fn system_library_dirs(&self) -> Vec<PathBuf> {
        self.system_libraries.clone()
    }
}
