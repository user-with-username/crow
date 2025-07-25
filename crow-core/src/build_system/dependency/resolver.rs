use super::types::{cmake, crow};
use crate::build_system;
use crate::config::{BuildProfile, CrowDependencyBuild, Dependency, ToolchainConfig};
use anyhow::Context;
use crow_utils::logger::{LogLevel, Logger};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct DependencyBuildOutput {
    pub lib_name: String,
    pub library_path: PathBuf,
    pub library_dir: PathBuf,
    pub include_paths: Vec<String>,
}

pub trait DependencyResolver {
    fn resolve_dependencies(
        dependencies: &HashMap<String, Dependency>,
        toolchain: &ToolchainConfig,
        current_profile: &str,
        profile_config: &BuildProfile,
        global_deps: bool,
        logger: Logger,
    ) -> anyhow::Result<(
        HashMap<String, PathBuf>,
        HashMap<String, DependencyBuildOutput>,
    )>;

    fn copy_local_dependency(
        name: &str,
        local_path_orig: &Path,
        global_local_dep_target_path: &Path,
        logger: Logger,
    ) -> anyhow::Result<()>;
}

impl DependencyResolver for build_system::BuildSystem {
    fn resolve_dependencies(
        dependencies: &HashMap<String, Dependency>,
        toolchain: &ToolchainConfig,
        current_profile: &str,
        profile_config: &BuildProfile,
        global_deps: bool,
        logger: Logger,
    ) -> anyhow::Result<(
        HashMap<String, PathBuf>,
        HashMap<String, DependencyBuildOutput>,
    )> {
        if dependencies.is_empty() {
            return Ok((HashMap::new(), HashMap::new()));
        }

        logger.log(LogLevel::Bold, "Checking dependencies...", 1);
        let deps_download_dir = crow_utils::environment::Environment::deps_dir(global_deps);
        std::fs::create_dir_all(&deps_download_dir)?;

        let original_cwd = std::env::current_dir()?;
        let mut downloaded_paths = HashMap::new();
        let mut dep_build_outputs = HashMap::new();

        let has_git_deps = dependencies
            .values()
            .any(|dep| matches!(dep, Dependency::Git { .. }));
        if has_git_deps {
            <build_system::BuildSystem as build_system::GitManager>::check_git_available()?;
        }

        for (name, dep) in dependencies {
            let dep_source_path: PathBuf;
            let crow_build_config: CrowDependencyBuild;

            match dep {
                Dependency::Git { git, branch, build } => {
                    let git_dep_target_path = deps_download_dir.join(name);

                    if git_dep_target_path.exists() {
                        if logger.verbose {
                            logger.log(
                                LogLevel::Dim,
                                &format!("Dependency '{name}' exists. Pulling updates..."),
                                1,
                            );
                        } else {
                            logger.log(LogLevel::Info, &format!("[UPDATING] {name} ({})", git), 2);
                        }
                        <build_system::BuildSystem as build_system::GitManager>::git_pull(
                            &git_dep_target_path,
                            &logger.clone(),
                        )?;
                    } else {
                        if logger.verbose {
                            logger.log(
                                LogLevel::Dim,
                                &format!("Cloning new dependency '{name}' from {}", git),
                                1,
                            );
                        } else {
                            logger.log(
                                LogLevel::Custom("\x1b[32m"),
                                &format!("[CLONING] {name} ({})", git),
                                2,
                            );
                        }
                        <build_system::BuildSystem as build_system::GitManager>::git_clone(
                            git,
                            branch,
                            &git_dep_target_path,
                            &logger.clone(),
                        )?;
                    }
                    dep_source_path = git_dep_target_path;
                    crow_build_config =
                        CrowDependencyBuild::infer_defaults(&dep_source_path, name, build.clone());
                }
                Dependency::Path { path, build } => {
                    let local_path_orig = PathBuf::from(path);
                    if !local_path_orig.exists() {
                        anyhow::bail!(
                            "Local dependency path '{}' for '{}' does not exist.",
                            local_path_orig.display(),
                            name
                        );
                    }

                    if global_deps {
                        let global_local_dep_target_path = deps_download_dir.join(name);
                        Self::copy_local_dependency(
                            name,
                            &local_path_orig,
                            &global_local_dep_target_path,
                            logger.clone(),
                        )?;
                        dep_source_path = global_local_dep_target_path;
                    } else {
                        if logger.verbose {
                            logger.log(
                                LogLevel::Dim,
                                &format!(
                                    "Using local dependency '{name}' from {}",
                                    local_path_orig.display()
                                ),
                                1,
                            );
                        } else {
                            logger.log(
                                LogLevel::Info,
                                &format!("[LOCAL] {name} ({})", local_path_orig.display()),
                                2,
                            );
                        }
                        dep_source_path = local_path_orig;
                    }
                    crow_build_config =
                        CrowDependencyBuild::infer_defaults(&dep_source_path, name, build.clone());
                }
            };

            downloaded_paths.insert(name.clone(), dep_source_path.clone());

            let build_output_dir = dep_source_path.join("_crow_build").join(current_profile);
            let lib_name_str = &crow_build_config.lib_name;

            let expected_lib_path = <build_system::builder::BuildSystem as build_system::ToolchainExecutor>::find_library_file(
                &build_output_dir,
                lib_name_str,
                &crow_build_config.output_type,
            );

            if let Some(lib_path) = expected_lib_path {
                logger.log(
                    LogLevel::Info,
                    &format!("[CACHED] Dependency '{name}' (profile: {current_profile})."),
                    2,
                );
                if logger.verbose {
                    logger.log(
                        LogLevel::Dim,
                        &format!("Library found at: {}", lib_path.display()),
                        2,
                    );
                }

                let mut include_paths = vec![".".to_string()];
                if dep_source_path.join("include").exists() {
                    include_paths.push("include".to_string());
                }

                let output = DependencyBuildOutput {
                    lib_name: lib_name_str.to_string(),
                    library_path: lib_path.clone(),
                    library_dir: lib_path.parent().unwrap().to_path_buf(),
                    include_paths,
                };
                dep_build_outputs.insert(name.clone(), output);
                continue;
            }

            logger.log(
                LogLevel::Bold,
                &format!("Building dependency '{name}'..."),
                1,
            );
            std::env::set_current_dir(&dep_source_path)?;

            let build_output = match crow_build_config.build_system {
                Some(crate::config::BuildSystemType::Cmake) => cmake::CmakeDependency::build(
                    name,
                    toolchain,
                    &crow_build_config,
                    current_profile,
                    profile_config,
                    logger.clone(),
                ),
                Some(crate::config::BuildSystemType::Crow) => crow::CrowDependency::build(
                    name,
                    &dep_source_path,
                    &crow_build_config,
                    current_profile,
                    global_deps,
                    logger.clone(),
                ),
                None => anyhow::bail!("Build system for dependency '{}' was not inferred.", name),
            }?;

            dep_build_outputs.insert(name.clone(), build_output);
            std::env::set_current_dir(&original_cwd)?;
            logger.log(
                LogLevel::Bold,
                &format!("Finished building dependency '{name}'."),
                1,
            );
        }
        logger.log(LogLevel::Bold, "Dependencies checked.", 1);
        Ok((downloaded_paths, dep_build_outputs))
    }

    fn copy_local_dependency(
        name: &str,
        local_path_orig: &Path,
        global_local_dep_target_path: &Path,
        logger: Logger,
    ) -> anyhow::Result<()> {
        if global_local_dep_target_path.exists() {
            if logger.verbose {
                logger.log(
                    LogLevel::Dim,
                    &format!(
                        "Global local dependency '{name}' already exists at {}. Overwriting...",
                        global_local_dep_target_path.display()
                    ),
                    1,
                );
            } else {
                logger.log(
                    LogLevel::Warn,
                    &format!("[OVERWRITING] Global local dependency '{name}'"),
                    1,
                );
            }
            std::fs::remove_dir_all(global_local_dep_target_path)?;
        } else {
            if logger.verbose {
                logger.log(
                    LogLevel::Dim,
                    &format!(
                        "Copying local dependency '{name}' from {} to {}",
                        local_path_orig.display(),
                        global_local_dep_target_path.display()
                    ),
                    1,
                );
            } else {
                logger.log(
                    LogLevel::Success,
                    &format!("[COPYING] Local dependency '{name}' to global cache"),
                    1,
                );
            }
        }

        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;
        fs_extra::dir::copy(local_path_orig, global_local_dep_target_path, &options).with_context(
            || {
                format!(
                    "Failed to copy dependency '{}' (path: {}) to global cache '{}'",
                    name,
                    local_path_orig.display(),
                    global_local_dep_target_path.display()
                )
            },
        )?;
        Ok(())
    }
}