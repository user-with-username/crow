use super::*;
use crate::build_system;
use crate::config::BuildSystemType;
use crate::config::{BuildProfile, CrowDependencyBuild, Dependency, ToolchainConfig};
use anyhow::Context;
use crow_utils::logger::Logger;
use crow_utils::logger::INDENT_LEVEL_1;
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
        verbose: bool,
        global_deps: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<(
        HashMap<String, PathBuf>,
        HashMap<String, DependencyBuildOutput>,
    )>;

    fn build_cmake_dependency(
        name: &str,
        toolchain: &ToolchainConfig,
        config: &CrowDependencyBuild,
        profile: &str,
        profile_config: &BuildProfile,
        verbose: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<DependencyBuildOutput>;

    fn build_crow_dependency(
        name: &str,
        dep_source_path: &Path,
        crow_build_config: &CrowDependencyBuild,
        current_profile: &str,
        verbose: bool,
        global_deps: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<DependencyBuildOutput>;

    fn copy_local_dependency(
        name: &str,
        local_path_orig: &Path,
        global_local_dep_target_path: &Path,
        verbose: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<()>;
}

impl DependencyResolver for BuildSystem {
    fn resolve_dependencies(
        dependencies: &HashMap<String, Dependency>,
        toolchain: &ToolchainConfig,
        current_profile: &str,
        profile_config: &BuildProfile,
        verbose: bool,
        global_deps: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<(
        HashMap<String, PathBuf>,
        HashMap<String, DependencyBuildOutput>,
    )> {
        if dependencies.is_empty() {
            return Ok((HashMap::new(), HashMap::new()));
        }

        logger.bold("Checking dependencies...");
        let deps_download_dir = crow_utils::environment::Environment::deps_dir(global_deps);
        std::fs::create_dir_all(&deps_download_dir)?;

        let original_cwd = std::env::current_dir()?;
        let mut downloaded_paths = HashMap::new();
        let mut dep_build_outputs = HashMap::new();

        let has_git_deps = dependencies
            .values()
            .any(|dep| matches!(dep, Dependency::Git { .. }));
        if has_git_deps {
            <BuildSystem as GitManager>::check_git_available()?;
        }

        for (name, dep) in dependencies {
            let dep_source_path: PathBuf;
            let crow_build_config: CrowDependencyBuild;

            match dep {
                Dependency::Git { git, branch, build } => {
                    let git_dep_target_path = deps_download_dir.join(name);

                    if git_dep_target_path.exists() {
                        if verbose {
                            logger.dim(&format!("Dependency '{name}' exists. Pulling updates..."));
                        } else {
                            logger.custom(
                                logger.colors.cyan,
                                &format!("{} [UPDATING] {name} ({})", INDENT_LEVEL_1, git),
                            );
                        }
                        <BuildSystem as GitManager>::git_pull(
                            &git_dep_target_path,
                            verbose,
                            logger,
                        )?;
                    } else {
                        if verbose {
                            logger.dim(&format!("Cloning new dependency '{name}' from {}", git));
                        } else {
                            logger.custom(
                                logger.colors.green,
                                &format!("{} [CLONING] {name} ({})", INDENT_LEVEL_1, git),
                            );
                        }
                        <BuildSystem as GitManager>::git_clone(
                            git,
                            branch,
                            &git_dep_target_path,
                            verbose,
                            logger,
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
                            verbose,
                            logger,
                        )?;
                        dep_source_path = global_local_dep_target_path;
                    } else {
                        if verbose {
                            logger.dim(&format!(
                                "Using local dependency '{name}' from {}",
                                local_path_orig.display()
                            ));
                        } else {
                            logger.custom(
                                logger.colors.cyan,
                                &format!(
                                    "{} [LOCAL] {name} ({})",
                                    INDENT_LEVEL_1,
                                    local_path_orig.display()
                                ),
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

            let expected_lib_path = <builder::BuildSystem as ToolchainExecutor>::find_library_file(
                &build_output_dir,
                lib_name_str,
                &crow_build_config.output_type,
            );

            if let Some(lib_path) = expected_lib_path {
                logger.custom(
                    logger.colors.cyan,
                    &format!(
                        "{} [CACHED] Dependency '{name}' (profile: {current_profile}).",
                        INDENT_LEVEL_1
                    ),
                );
                if verbose {
                    logger.dim_level2(&format!("Library found at: {}", lib_path.display()));
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

            logger.bold(&format!("Building dependency '{name}'..."));
            std::env::set_current_dir(&dep_source_path)?;

            let build_output = match crow_build_config.build_system {
                Some(BuildSystemType::Cmake) => Self::build_cmake_dependency(
                    name,
                    toolchain,
                    &crow_build_config,
                    current_profile,
                    profile_config,
                    verbose,
                    logger,
                ),
                Some(BuildSystemType::Crow) => Self::build_crow_dependency(
                    name,
                    &dep_source_path,
                    &crow_build_config,
                    current_profile,
                    verbose,
                    global_deps,
                    logger,
                ),
                None => anyhow::bail!("Build system for dependency '{}' was not inferred.", name),
            }?;

            dep_build_outputs.insert(name.clone(), build_output);
            std::env::set_current_dir(&original_cwd)?;
            logger.bold(&format!("Finished building dependency '{name}'."));
        }
        logger.bold("Dependencies checked.");
        Ok((downloaded_paths, dep_build_outputs))
    }

    fn copy_local_dependency(
        name: &str,
        local_path_orig: &Path,
        global_local_dep_target_path: &Path,
        verbose: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<()> {
        if global_local_dep_target_path.exists() {
            if verbose {
                logger.dim(&format!(
                    "Global local dependency '{name}' already exists at {}. Overwriting...",
                    global_local_dep_target_path.display()
                ));
            } else {
                logger.warn(&format!(
                    "{} [OVERWRITING] Global local dependency '{name}'",
                    INDENT_LEVEL_1
                ));
            }
            std::fs::remove_dir_all(global_local_dep_target_path)?;
        } else {
            if verbose {
                logger.dim(&format!(
                    "Copying local dependency '{name}' from {} to {}",
                    local_path_orig.display(),
                    global_local_dep_target_path.display()
                ));
            } else {
                logger.custom(
                    logger.colors.green,
                    &format!(
                        "{} [COPYING] Local dependency '{name}' to global cache",
                        INDENT_LEVEL_1
                    ),
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

    fn build_cmake_dependency(
        name: &str,
        toolchain: &ToolchainConfig,
        config: &CrowDependencyBuild,
        profile: &str,
        profile_config: &BuildProfile,
        verbose: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<DependencyBuildOutput> {
        let dep_source_dir = std::env::current_dir()?;
        let build_dir = dep_source_dir.join("_crow_build").join(profile);
        let lib_name = config.lib_name.clone();

        std::fs::create_dir_all(&build_dir)?;

        let build_type = if profile == "release" {
            "Release"
        } else {
            "Debug"
        };

        let mut cxx_flags = vec![format!("-O{}", profile_config.opt_level)];
        if profile_config.lto {
            cxx_flags.push("-flto".to_string());
        }

        BuildSystem::handle_pch_generation(
            name,
            toolchain,
            config,
            profile_config,
            verbose,
            &build_dir,
            &mut cxx_flags,
            logger,
        )?;

        let cxx_flags_str = cxx_flags.join(" ");

        BuildSystem::run_cmake_configure(
            name,
            &dep_source_dir,
            &build_dir,
            build_type,
            toolchain,
            &cxx_flags_str,
            &config.cmake_options,
            verbose,
            logger,
        )?;
        BuildSystem::run_cmake_build(name, &build_dir, build_type, verbose, logger)?;

        let library_path =
            <build_system::builder::BuildSystem as ToolchainExecutor>::find_library_file(
                &build_dir,
                &lib_name,
                &config.output_type,
            )
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not find library for '{}' after build in {}",
                    name,
                    build_dir.display()
                )
            })?;

        let mut include_paths = vec![".".to_string()];
        if dep_source_dir.join("include").exists() {
            include_paths.push("include".to_string());
        }

        Ok(DependencyBuildOutput {
            lib_name,
            library_path: std::fs::canonicalize(&library_path)?,
            library_dir: std::fs::canonicalize(library_path.parent().unwrap())?,
            include_paths,
        })
    }

    fn build_crow_dependency(
        name: &str,
        dep_source_path: &Path,
        crow_build_config: &CrowDependencyBuild,
        current_profile: &str,
        verbose: bool,
        global_deps: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<DependencyBuildOutput> {
        let dep_crow_toml = dep_source_path.join("crow.toml");
        if !dep_crow_toml.exists() {
            anyhow::bail!(
                "Dependency '{name}' is configured for CRow build, but no `crow.toml` found."
            );
        }

        let dep_config = Config::load(&dep_crow_toml)?;
        let mut dep_package_config = dep_config.package.clone();
        dep_package_config.output_type = crow_build_config.output_type.clone();

        let dep_build_system =
            BuildSystem::new(dep_config, current_profile, verbose, global_deps, logger)?;
        dep_build_system.build_internal(Some(1), Some(&dep_package_config))
    }
}
