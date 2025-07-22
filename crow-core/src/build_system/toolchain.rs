use super::*;
use crate::build_system::cache::CacheManager;
use crate::config::OutputType;
use crow_utils::logger::{LogLevel, Logger};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub trait ToolchainExecutor {
    fn compile_with_args(
        compiler: &str,
        args: &[std::ffi::OsString],
        source: &Path,
        output: &Path,
        incremental: bool,
        verbose: bool,
        logger: &Logger,
    ) -> anyhow::Result<(PathBuf, u64)>;
    fn link_executable(&self, objects: &[PathBuf], output: &Path) -> anyhow::Result<()>;
    fn archive_static_library(&self, objects: &[PathBuf], output: &Path) -> anyhow::Result<()>;
    fn link_shared_library(&self, objects: &[PathBuf], output: &Path) -> anyhow::Result<()>;
    fn format_static_lib_name(name: &str) -> String;
    fn format_shared_lib_name(name: &str) -> String;
    fn set_executable_permissions(path: &Path) -> anyhow::Result<()>;
    fn find_library_file(dir: &Path, name: &str, output_type: &OutputType) -> Option<PathBuf>;
    fn find_library_file_recursive(dir: &Path, patterns: &[String]) -> Option<PathBuf>;
}

impl ToolchainExecutor for BuildSystem {
    fn compile_with_args(
        compiler: &str,
        args: &[std::ffi::OsString],
        source: &Path,
        output: &Path,
        incremental: bool,
        verbose: bool,
        logger: &Logger,
    ) -> anyhow::Result<(PathBuf, u64)> {
        let mut cmd = Command::new(compiler);
        cmd.args(args);

        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let output_res = cmd.output()?;

        if !output_res.status.success() {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            logger.log(
                LogLevel::Error,
                &format!(
                    "Compiler error for {}:\n{} {}",
                    source.display(),
                    stdout_output,
                    stderr_output
                ),
                0,
            );
            anyhow::bail!("Compiler error for {}", source.display());
        } else if verbose {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            if !stdout_output.is_empty() {
                logger.log(
                    LogLevel::Dim,
                    &format!("Compiler stdout for {}:", source.display()),
                    2,
                );
                logger.log((), &stdout_output, 2);
            }
            if !stderr_output.is_empty() {
                logger.log(
                    LogLevel::Dim,
                    &format!("Compiler stderr for {}:", source.display()),
                    2,
                );
                logger.log((), &stderr_output, 2);
            }
        }

        let deps_hash = if incremental {
            BuildCache::parse_dep_file(&output.with_extension("d"))
                .and_then(|deps| BuildCache::compute_deps_hash(&deps))
                .unwrap_or(0)
        } else {
            0
        };
        Ok((output.to_path_buf(), deps_hash))
    }

    fn link_executable(&self, objects: &[PathBuf], output: &Path) -> anyhow::Result<()> {
        if self.verbose {
            self.logger.log(
                LogLevel::Dim,
                &format!("Linking executable: {}", output.display()),
                1,
            );
        }
        let mut cmd = Command::new(&self.toolchain.linker);
        objects.iter().for_each(|o| {
            cmd.arg(o);
        });
        self.toolchain.linker_flags.iter().for_each(|f| {
            cmd.arg(f);
        });

        if self.profile_config.lto {
            cmd.arg("-flto");
            cmd.arg(format!("-O{}", self.profile_config.opt_level));
        }

        for lib_dir in &self.package_config.lib_dirs {
            if self.verbose {
                self.logger.log(
                    LogLevel::Dim,
                    &format!("-L '{}' (from crow.toml)", lib_dir),
                    2,
                );
            }
            cmd.arg(format!("-L{}", lib_dir));
        }
        for (name, build_output) in &self.dep_build_outputs {
            if self.verbose {
                self.logger.log(
                    LogLevel::Dim,
                    &format!(
                        "-L '{}' (from dependency '{name}')",
                        build_output.library_dir.display()
                    ),
                    2,
                );
            }
            cmd.arg(format!("-L{}", build_output.library_dir.display()));
        }

        for lib in &self.package_config.libs {
            if self.verbose {
                self.logger
                    .log(LogLevel::Dim, &format!("-l '{}' (from crow.toml)", lib), 2);
            }
            cmd.arg(format!("-l{}", lib));
        }
        for (name, build_output) in &self.dep_build_outputs {
            if self.verbose {
                self.logger.log(
                    LogLevel::Dim,
                    &format!("-l '{}' (from dependency '{name}')", build_output.lib_name),
                    2,
                );
            }
            cmd.arg(format!("-l{}", build_output.lib_name));
        }

        cmd.arg("-o").arg(output);

        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let output_res = cmd.output()?;
        if !output_res.status.success() {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            self.logger.log(
                LogLevel::Error,
                &format!("Linking failed:\n{} {}", stdout_output, stderr_output),
                0,
            );
            anyhow::bail!("Linking failed:\n{} {}", stdout_output, stderr_output);
        } else if self.verbose {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            if !stdout_output.is_empty() {
                self.logger.log(LogLevel::Dim, "Linker stdout:", 2);
                self.logger.log((), &stdout_output, 2);
            }
            if !stderr_output.is_empty() {
                self.logger.log(LogLevel::Dim, "Linker stderr:", 2);
                self.logger.log((), &stderr_output, 2);
            }
        }
        Ok(())
    }

    fn archive_static_library(&self, objects: &[PathBuf], output: &Path) -> anyhow::Result<()> {
        if self.verbose {
            self.logger.log(
                LogLevel::Dim,
                &format!("Archiving static library: {}", output.display()),
                0,
            );
        }
        let mut cmd = Command::new(&self.toolchain.archiver);
        self.toolchain.archiver_flags.iter().for_each(|f| {
            cmd.arg(f);
        });
        cmd.arg(output);
        objects.iter().for_each(|o| {
            cmd.arg(o);
        });

        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let output_res = cmd.output()?;
        if !output_res.status.success() {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            self.logger.log(
                LogLevel::Error,
                &format!("Archiving failed:\n{} {}", stdout_output, stderr_output),
                0,
            );
            anyhow::bail!("Archiving failed:\n{} {}", stdout_output, stderr_output);
        } else if self.verbose {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            if !stdout_output.is_empty() {
                self.logger.log(LogLevel::Dim, "Archiver stdout:", 2);
                self.logger.log((), &stdout_output, 2);
            }
            if !stderr_output.is_empty() {
                self.logger.log(LogLevel::Dim, "Archiver stderr:", 2);
                self.logger.log((), &stderr_output, 2);
            }
        }
        Ok(())
    }

    fn link_shared_library(&self, objects: &[PathBuf], output: &Path) -> anyhow::Result<()> {
        if self.verbose {
            self.logger.log(
                LogLevel::Dim,
                &format!("Linking shared library: {}", output.display()),
                0,
            );
        }
        let mut cmd = Command::new(&self.toolchain.linker);
        cmd.arg("-shared");
        objects.iter().for_each(|o| {
            cmd.arg(o);
        });
        self.toolchain.linker_flags.iter().for_each(|f| {
            cmd.arg(f);
        });

        if self.profile_config.lto {
            cmd.arg("-flto");
            cmd.arg(format!("-O{}", self.profile_config.opt_level));
        }

        for lib_dir in &self.package_config.lib_dirs {
            if self.verbose {
                self.logger.log(
                    LogLevel::Dim,
                    &format!("-L '{}' (from crow.toml)", lib_dir),
                    2,
                );
            }
            cmd.arg(format!("-L{}", lib_dir));
        }
        for (name, build_output) in &self.dep_build_outputs {
            if self.verbose {
                self.logger.log(
                    LogLevel::Dim,
                    &format!(
                        "-L '{}' (from dependency '{name}')",
                        build_output.library_dir.display()
                    ),
                    2,
                );
            }
            cmd.arg(format!("-L{}", build_output.library_dir.display()));
        }

        for lib in &self.package_config.libs {
            if self.verbose {
                self.logger
                    .log(LogLevel::Dim, &format!("-l '{}' (from crow.toml)", lib), 2);
            }
            cmd.arg(format!("-l{}", lib));
        }
        for (name, build_output) in &self.dep_build_outputs {
            if self.verbose {
                self.logger.log(
                    LogLevel::Dim,
                    &format!("-l '{}' (from dependency '{name}')", build_output.lib_name),
                    2,
                );
            }
            cmd.arg(format!("-l{}", build_output.lib_name));
        }

        cmd.arg("-o").arg(output);

        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let output_res = cmd.output()?;
        if !output_res.status.success() {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            self.logger.log(
                LogLevel::Error,
                &format!(
                    "Linking shared library failed:\n{} {}",
                    stdout_output, stderr_output
                ),
                0,
            );
            anyhow::bail!(
                "Linking shared library failed:\n{} {}",
                stdout_output,
                stderr_output
            );
        } else if self.verbose {
            let stdout_output = String::from_utf8_lossy(&output_res.stdout);
            let stderr_output = String::from_utf8_lossy(&output_res.stderr);
            if !stdout_output.is_empty() {
                self.logger.log(LogLevel::Dim, "Shared Linker stdout:", 2);
                self.logger.log((), &stdout_output, 2);
            }
            if !stderr_output.is_empty() {
                self.logger.log(LogLevel::Dim, "Shared Linker stderr:", 2);
                self.logger.log((), &stderr_output, 2);
            }
        }
        Ok(())
    }

    fn format_static_lib_name(name: &str) -> String {
        if cfg!(windows) {
            format!("{}.lib", name)
        } else {
            format!("lib{}.a", name)
        }
    }

    fn format_shared_lib_name(name: &str) -> String {
        if cfg!(windows) {
            format!("{}.dll", name)
        } else if cfg!(target_os = "macos") {
            format!("lib{}.dylib", name)
        } else {
            format!("lib{}.so", name)
        }
    }

    #[cfg(unix)]
    fn set_executable_permissions(path: &Path) -> anyhow::Result<()> {
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_executable_permissions(_path: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    fn find_library_file(dir: &Path, name: &str, output_type: &OutputType) -> Option<PathBuf> {
        if !dir.exists() {
            return None;
        }

        let patterns = match output_type {
            OutputType::StaticLib => {
                if cfg!(windows) {
                    vec![format!("{}.lib", name)]
                } else {
                    vec![format!("lib{}.a", name)]
                }
            }
            OutputType::SharedLib => {
                if cfg!(windows) {
                    vec![format!("{}.dll", name), format!("{}.lib", name)]
                } else if cfg!(target_os = "macos") {
                    vec![format!("lib{}.dylib", name)]
                } else {
                    vec![format!("lib{}.so", name)]
                }
            }
            OutputType::Executable => {
                println!("exe");
                return None;
            }
        };

        Self::find_library_file_recursive(dir, &patterns)
    }

    fn find_library_file_recursive(dir: &Path, patterns: &[String]) -> Option<PathBuf> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let file_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default();
                    if file_name.starts_with('.')
                        || file_name == "CMakeFiles"
                        || file_name == "cmake"
                        || file_name == "doc"
                        || file_name == "examples"
                        || file_name == "tests"
                        || file_name == "test"
                        || file_name == "support"
                    {
                        continue;
                    }
                    if let Some(found) = Self::find_library_file_recursive(&path, patterns) {
                        return Some(found);
                    }
                } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if patterns.iter().any(|p| p == file_name) {
                        return Some(path);
                    }
                }
            }
        }
        None
    }
}
