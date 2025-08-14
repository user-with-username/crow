pub mod hooks;
pub mod incremental;

use super::*;
use crate::config::{CrowDependencyBuild, OutputType};
use crate::utils;
use crow_utils::LogLevel;
use hooks::Executor;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc;
use std::{env, thread};

pub struct BuildSystem {
    pub config: Config,
    pub toolchain: ToolchainConfig,
    pub profile_config: BuildProfile,
    pub package_config: PackageConfig,
    pub profile_name: String,
    pub global_deps: bool,
    pub downloaded_deps_paths: HashMap<String, PathBuf>,
    pub dep_build_outputs: HashMap<String, DependencyBuildOutput>,
    pub logger: Logger,
}

impl BuildSystem {
    pub fn new(
        config: Config,
        profile_name: &str,
        global_deps: bool,
        logger: Logger,
    ) -> anyhow::Result<Self> {
        let (package_config, toolchain, profile_config) =
            BuildSystem::resolve_config(&config, profile_name, logger.clone())?;

        let (downloaded_deps_paths, dep_build_outputs) = BuildSystem::resolve_dependencies(
            &config.dependencies,
            &toolchain,
            profile_name,
            &profile_config,
            global_deps,
            logger.clone(),
        )?;

        Ok(Self {
            config,
            toolchain,
            profile_config,
            package_config,
            profile_name: profile_name.to_string(),
            global_deps,
            downloaded_deps_paths,
            dep_build_outputs,
            logger,
        })
    }

    pub fn resolve_config(
        config: &Config,
        profile_name: &str,
        logger: Logger,
    ) -> anyhow::Result<(PackageConfig, ToolchainConfig, BuildProfile)> {
        let profile_config_base = config
            .profiles
            .as_ref()
            .and_then(|p| p.get(profile_name))
            .ok_or_else(|| {
                anyhow::anyhow!("Profile '{}' not found in configuration.", profile_name)
            })?
            .clone();

        let mut toolchain = config.toolchain.clone();
        let mut profile_config = profile_config_base.clone();
        let mut package_config = config.package.clone();

        toolchain
            .hooks
            .pre_execute
            .as_ref()
            .map(|hooks| Executor::execute_hooks(hooks, logger.clone()))
            .transpose()?;

        let current_arch = env::consts::ARCH;
        let current_os = env::consts::OS;

        let best_target_overrides = config
            .targets
            .iter()
            .filter_map(|(target_name, target)| {
                let os_match = target.os.as_ref().map_or(true, |os| os == current_os);
                let arch_match = target
                    .arch
                    .as_ref()
                    .map_or(true, |arch| arch == current_arch);

                if os_match && arch_match {
                    let score =
                        target.os.as_ref().map_or(0, |_| 1) + target.arch.as_ref().map_or(0, |_| 2);
                    Some((score, target_name, target))
                } else {
                    None
                }
            })
            .max_by_key(|(score, _, _)| *score)
            .map(|(_, _, target)| target.clone());

        best_target_overrides
            .as_ref()
            .map(|target_override| -> anyhow::Result<()> {
                target_override
                    .hooks
                    .pre_execute
                    .as_ref()
                    .map(|hooks| Executor::execute_hooks(hooks, logger.clone()))
                    .transpose()?;

                target_override
                    .toolchain
                    .as_ref()
                    .map(|toolchain_override| -> anyhow::Result<()> {
                        toolchain_override
                            .hooks
                            .pre_execute
                            .as_ref()
                            .map(|hooks| Executor::execute_hooks(hooks, logger.clone()))
                            .transpose()?;

                        toolchain_override
                            .compiler
                            .as_ref()
                            .map(|compiler| toolchain.compiler.clone_from(compiler));
                        toolchain_override
                            .compiler_flags
                            .as_ref()
                            .map(|flags| toolchain.compiler_flags.clone_from(flags));
                        toolchain_override
                            .linker
                            .as_ref()
                            .map(|linker| toolchain.linker.clone_from(linker));
                        toolchain_override
                            .linker_flags
                            .as_ref()
                            .map(|flags| toolchain.linker_flags.clone_from(flags));
                        toolchain_override
                            .archiver
                            .as_ref()
                            .map(|archiver| toolchain.archiver.clone_from(archiver));
                        toolchain_override
                            .archiver_flags
                            .as_ref()
                            .map(|flags| toolchain.archiver_flags.clone_from(flags));

                        toolchain_override
                            .hooks
                            .post_execute
                            .as_ref()
                            .map(|hooks| Executor::execute_hooks(hooks, logger.clone()))
                            .transpose()?;

                        Ok::<(), anyhow::Error>(())
                    })
                    .transpose()?;

                target_override
                    .name
                    .as_ref()
                    .map(|name| package_config.name.clone_from(name));
                target_override
                    .output_type
                    .as_ref()
                    .map(|ot| package_config.output_type.clone_from(ot));
                target_override
                    .sources
                    .as_ref()
                    .map(|sources| package_config.sources.clone_from(sources));
                target_override
                    .includes
                    .as_ref()
                    .map(|includes| package_config.includes.clone_from(includes));
                target_override
                    .libs
                    .as_ref()
                    .map(|libs| package_config.libs.clone_from(libs));
                target_override
                    .lib_dirs
                    .as_ref()
                    .map(|dirs| package_config.lib_dirs.clone_from(dirs));

                target_override
                    .opt_level
                    .map(|level| profile_config.opt_level = level);
                target_override
                    .defines
                    .as_ref()
                    .map(|defines| profile_config.defines.clone_from(defines));
                target_override.lto.map(|lto| profile_config.lto = lto);
                target_override
                    .flags
                    .as_ref()
                    .map(|flags| profile_config.flags.clone_from(flags));
                target_override
                    .incremental
                    .map(|inc| profile_config.incremental = inc);

                target_override
                    .hooks
                    .post_execute
                    .as_ref()
                    .map(|hooks| Executor::execute_hooks(hooks, logger.clone()))
                    .transpose()?;

                Ok::<(), anyhow::Error>(())
            })
            .transpose()?;

        toolchain
            .hooks
            .post_execute
            .as_ref()
            .map(|hooks| Executor::execute_hooks(hooks, logger.clone()))
            .transpose()?;

        if logger.verbose && best_target_overrides.is_none() {
            logger.log(
                LogLevel::Dim,
                &format!("Building for `{}` with default settings", env::consts::ARCH),
                2,
            );
        }

        Ok((package_config, toolchain, profile_config))
    }

    pub fn build_internal(
        &self,
        jobs: Option<usize>,
        override_package_config: Option<&PackageConfig>,
    ) -> anyhow::Result<DependencyBuildOutput> {
        let package_config = override_package_config.unwrap_or(&self.package_config);

        if override_package_config.is_none() {
            self.logger.log(
                LogLevel::Bold,
                &format!(
                    "Building package `{}` (profile: {}, type: {:?})...",
                    package_config.name, self.profile_name, package_config.output_type
                ),
                1,
            );
        }

        let build_dir = crow_utils::environment::Environment::build_dir().join(&self.profile_name);
        std::fs::create_dir_all(&build_dir)?;
        let cwd = std::env::current_dir()?;

        let object_files = if self.profile_config.incremental {
            let incremental_builder = crate::build_system::IncrementalBuilder::new(self)?;
            incremental_builder.build(jobs, package_config)?
        } else {
            self.build_non_incremental(jobs, package_config)?
        };

        let output_path = match package_config.output_type {
            OutputType::Executable => {
                let exe_path = build_dir.join(&package_config.name);
                self.link_executable(&object_files, &exe_path)?;
                <BuildSystem as ToolchainExecutor>::set_executable_permissions(&exe_path)?;
                exe_path
            }
            OutputType::StaticLib => {
                let lib_path = build_dir.join(Self::format_static_lib_name(&package_config.name));
                self.archive_static_library(&object_files, &lib_path)?;
                lib_path
            }
            OutputType::SharedLib => {
                let lib_path = build_dir.join(Self::format_shared_lib_name(&package_config.name));
                self.link_shared_library(&object_files, &lib_path)?;
                lib_path
            }
        };

        if override_package_config.is_none() {
            self.logger.log(LogLevel::Success, "Build successful!", 1);
        }

        let build_output = DependencyBuildOutput {
            lib_name: package_config.name.clone(),
            library_path: cwd.join(&output_path),
            library_dir: cwd.join(&build_dir),
            include_paths: package_config.includes.clone(),
        };

        Ok(build_output)
    }

    pub fn build(&self, jobs: Option<usize>) -> anyhow::Result<PathBuf> {
        let build_output = self.build_internal(jobs, None)?;
        Ok(build_output.library_path)
    }

    fn build_non_incremental(
        &self,
        jobs: Option<usize>,
        package_config: &PackageConfig,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let build_dir = crow_utils::environment::Environment::build_dir().join(&self.profile_name);
        std::fs::create_dir_all(&build_dir)?;
        let sources = utils::find_source_files(package_config)?;

        let num_jobs = jobs.unwrap_or_else(|| {
            thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        });
        let pool = threadpool::ThreadPool::new(num_jobs);
        let (tx, rx) = mpsc::channel();
        let mut object_files = Vec::new();
        let mut had_errors = false;

        for source_path in &sources {
            let obj_path = build_dir.join(source_path.with_extension("o").file_name().unwrap());

            let tx = tx.clone();
            let compiler_path = self.toolchain.compiler.clone();
            let source_clone = source_path.clone();
            let obj_path_clone = obj_path.clone();
            let toolchain_clone = self.toolchain.clone();
            let profile_config_clone = self.profile_config.clone();
            let package_config_clone = package_config.clone();
            let downloaded_deps_paths_clone = self.downloaded_deps_paths.clone();
            let dep_build_outputs_clone = self.dep_build_outputs.clone();
            let logger_clone = self.logger.clone();

            let is_verbose = self.logger.verbose;

            pool.execute(move || {
                let args_for_thread = BuildSystem::build_compile_args_static(
                    &toolchain_clone,
                    &profile_config_clone,
                    &package_config_clone,
                    &downloaded_deps_paths_clone,
                    &dep_build_outputs_clone,
                    &source_clone,
                    &obj_path_clone,
                )
                .expect("Failed while building compile args in thread");

                let result = <BuildSystem as ToolchainExecutor>::compile_with_args(
                    &compiler_path,
                    &args_for_thread,
                    &source_clone,
                    &obj_path_clone,
                    false,
                    &logger_clone,
                );
                if is_verbose {
                    if let Ok(_) = &result {
                        logger_clone.log(
                            LogLevel::Custom("\x1b[32m"),
                            &format!("[COMPILED] {}", source_clone.display()),
                            2,
                        );
                    }
                }
                tx.send((source_clone, result)).unwrap();
            });
        }

        drop(tx);
        for (_, result) in rx.iter() {
            match result {
                Ok((obj_path, _)) => {
                    object_files.push(obj_path);
                }
                Err(_) => had_errors = true,
            }
        }

        if had_errors {
            anyhow::bail!("Compilation failed.");
        }

        Ok(object_files)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_compile_args_static(
        toolchain: &ToolchainConfig,
        profile: &BuildProfile,
        package: &PackageConfig,
        downloaded_deps_paths: &HashMap<String, PathBuf>,
        dep_build_outputs: &HashMap<String, DependencyBuildOutput>,
        source: &Path,
        output: &Path,
    ) -> anyhow::Result<Vec<std::ffi::OsString>> {
        let mut args = vec![
            std::ffi::OsString::from("-c"),
            source.as_os_str().to_os_string(),
            std::ffi::OsString::from("-o"),
            output.as_os_str().to_os_string(),
            std::ffi::OsString::from(format!("-O{}", profile.opt_level)),
        ];
        toolchain
            .compiler_flags
            .iter()
            .for_each(|f| args.push(f.into()));
        if profile.lto {
            args.push("-flto".into());
            args.push("-fuse-ld=lld".into());
        }
        profile.flags.iter().for_each(|f| args.push(f.into()));
        profile
            .defines
            .iter()
            .for_each(|d| args.push(format!("-D{}", d).into()));
        package
            .includes
            .iter()
            .for_each(|i| args.push(format!("-I{}", i).into()));

        for (dep_name, dep_root_path) in downloaded_deps_paths {
            if let Some(build_output) = dep_build_outputs.get(dep_name) {
                for relative_include in &build_output.include_paths {
                    let final_include_path = if Path::new(relative_include).is_absolute() {
                        PathBuf::from(relative_include)
                    } else {
                        dep_root_path.join(relative_include)
                    };
                    args.push(format!("-I{}", final_include_path.display()).into());
                }
            } else {
                args.push(format!("-I{}", dep_root_path.display()).into());
                let dep_include_path = dep_root_path.join("include");
                if dep_include_path.exists() {
                    args.push(format!("-I{}", dep_include_path.display()).into());
                }
            }
        }

        if profile.incremental {
            let dep_path = output.with_extension("d");
            args.push("-MMD".into());
            args.push("-MF".into());
            args.push(dep_path.into_os_string());
        }
        Ok(args)
    }

    pub fn build_compile_args(
        &self,
        source: &Path,
        output: &Path,
    ) -> anyhow::Result<Vec<std::ffi::OsString>> {
        Self::build_compile_args_static(
            &self.toolchain,
            &self.profile_config,
            &self.package_config,
            &self.downloaded_deps_paths,
            &self.dep_build_outputs,
            source,
            output,
        )
    }

    pub fn handle_pch_generation(
        name: &str,
        toolchain: &ToolchainConfig,
        config: &CrowDependencyBuild,
        profile_config: &BuildProfile,
        build_dir: &Path,
        cxx_flags: &mut Vec<String>,
        logger: Logger,
    ) -> anyhow::Result<()> {
        if !config.pch_headers.is_empty() {
            let pch_file_path = build_dir.join("crow_pch.h");
            let pch_output_path = build_dir.join("crow_pch.h.gch");
            let pch_content: String = config
                .pch_headers
                .iter()
                .map(|h| format!("#include <{}>\n", h))
                .collect();
            std::fs::write(&pch_file_path, pch_content)?;

            if logger.verbose {
                logger.log(LogLevel::Dim, &format!("Generating PCH for '{name}'..."), 1);
            }

            let mut pch_cmd = std::process::Command::new(&toolchain.compiler);
            pch_cmd
                .arg("-x")
                .arg("c++-header")
                .arg(&pch_file_path)
                .arg("-o")
                .arg(&pch_output_path)
                .arg("-std=c++17")
                .arg(format!("-O{}", profile_config.opt_level));

            pch_cmd.stderr(Stdio::piped());
            pch_cmd.stdout(Stdio::piped());

            let pch_output = pch_cmd.output()?;
            if !pch_output.status.success() {
                let stdout_output = String::from_utf8_lossy(&pch_output.stdout);
                let stderr_output = String::from_utf8_lossy(&pch_output.stderr);
                logger.log(
                    LogLevel::Error,
                    &format!(
                        "Failed while generating precompiled header for '{}':\n{} {}",
                        name, stdout_output, stderr_output
                    ),
                    1,
                );
                anyhow::bail!("Failed while generating precompiled header for '{}'", name);
            } else if logger.verbose {
                let stdout_output = String::from_utf8_lossy(&pch_output.stdout);
                let stderr_output = String::from_utf8_lossy(&pch_output.stderr);
                if !stdout_output.is_empty() {
                    logger.log(LogLevel::Dim, "PCH Compiler stdout:", 2);
                    logger.log(LogLevel::Info, &stdout_output, 2);
                }
                if !stderr_output.is_empty() {
                    logger.log(LogLevel::Dim, "PCH Compiler stderr:", 2);
                    logger.log(LogLevel::Info, &stderr_output, 2);
                }
            }
            cxx_flags.push(format!("-include {}", pch_file_path.display()));
        }
        Ok(())
    }

    pub fn run_cmake_configure(
        name: &str,
        dep_source_dir: &Path,
        build_dir: &Path,
        build_type: &str,
        toolchain: &ToolchainConfig,
        cxx_flags_str: &str,
        cmake_options: &[String],
        logger: Logger,
    ) -> anyhow::Result<()> {
        let cmake_cache = build_dir.join("CMakeCache.txt");
        if !cmake_cache.exists() {
            if logger.verbose {
                logger.log(
                    LogLevel::Dim,
                    &format!("Running initial CMake configure for '{name}'..."),
                    1,
                );
            }
            let mut cmake_cmd = std::process::Command::new("cmake");
            cmake_cmd
                .arg("-S")
                .arg(dep_source_dir)
                .arg("-B")
                .arg(build_dir)
                .arg(format!("-DCMAKE_BUILD_TYPE={}", build_type))
                .arg(format!("-DCMAKE_CXX_COMPILER={}", &toolchain.compiler))
                .arg(format!("-DCMAKE_CXX_FLAGS={}", cxx_flags_str))
                .arg("-DCMAKE_DEBUG_POSTFIX=")
                .arg("-DBUILD_TESTING=OFF")
                .arg("-DCMAKE_MSVC_RUNTIME_LIBRARY='MultiThreaded'")
                .arg("-DCMAKE_DISABLE_TESTING=ON");

            for opt in cmake_options {
                cmake_cmd.arg(opt);
            }

            if logger.verbose {
                logger.log(
                    LogLevel::Dim,
                    &format!("Running CMake configure command: {:?}", cmake_cmd),
                    2,
                );
            }

            cmake_cmd.stderr(Stdio::piped());
            cmake_cmd.stdout(Stdio::piped());

            let output = cmake_cmd.output()?;
            if !output.status.success() {
                let stdout_output = String::from_utf8_lossy(&output.stdout);
                let stderr_output = String::from_utf8_lossy(&output.stderr);
                logger.log(
                    LogLevel::Error,
                    &format!(
                        "CMake configure failed for dependency '{}':\n{} {}",
                        name, stdout_output, stderr_output
                    ),
                    1,
                );
                anyhow::bail!("Cmake failed while configuring dependency '{}'", name);
            } else if logger.verbose {
                let stdout_output = String::from_utf8_lossy(&output.stdout);
                let stderr_output = String::from_utf8_lossy(&output.stderr);
                if !stdout_output.is_empty() {
                    logger.log(LogLevel::Dim, "CMake configure stdout:", 2);
                    logger.log(LogLevel::Info, &stdout_output, 2);
                }
                if !stderr_output.is_empty() {
                    logger.log(LogLevel::Dim, "CMake configure stderr:", 2);
                    logger.log(LogLevel::Info, &stderr_output, 2);
                }
            }
        }
        Ok(())
    }

    pub fn run_cmake_build(
        name: &str,
        build_dir: &Path,
        build_type: &str,
        logger: Logger,
    ) -> anyhow::Result<()> {
        let mut build_cmd = std::process::Command::new("cmake");
        build_cmd
            .arg("--build")
            .arg(build_dir)
            .arg("--config")
            .arg(build_type);
        if logger.verbose {
            logger.log(
                LogLevel::Dim,
                &format!("Running CMake build command: {:?}", build_cmd),
                2,
            );
        }

        build_cmd.stderr(Stdio::piped());
        build_cmd.stdout(Stdio::piped());

        let output = build_cmd.output()?;
        if !output.status.success() {
            let stdout_output = String::from_utf8_lossy(&output.stdout);
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            logger.log(
                LogLevel::Error,
                &format!(
                    "CMake build failed for dependency '{}':\n{} {}",
                    name, stdout_output, stderr_output
                ),
                1,
            );
            anyhow::bail!("CMake build failed for dependency '{}'", name);
        } else if logger.verbose {
            let stdout_output = String::from_utf8_lossy(&output.stdout);
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            if !stdout_output.is_empty() {
                logger.log(LogLevel::Dim, "CMake build stdout:", 2);
                logger.log(LogLevel::Info, &stdout_output, 2);
            }
            if !stderr_output.is_empty() {
                logger.log(LogLevel::Dim, "CMake build stderr:", 2);
                logger.log(LogLevel::Info, &stderr_output, 2);
            }
        }
        Ok(())
    }
}