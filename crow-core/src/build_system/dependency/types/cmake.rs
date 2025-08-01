use crate::config::{BuildProfile, CrowDependencyBuild, ToolchainConfig};
use crate::{build_system, DependencyBuildOutput};
use anyhow::anyhow;
use crow_utils::logger::Logger;

pub struct CmakeDependency;

impl CmakeDependency {
    pub fn build(
        name: &str,
        toolchain: &ToolchainConfig,
        config: &CrowDependencyBuild,
        profile: &str,
        profile_config: &BuildProfile,
        logger: Logger,
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

        build_system::BuildSystem::handle_pch_generation(
            name,
            toolchain,
            config,
            profile_config,
            &build_dir,
            &mut cxx_flags,
            logger.clone(),
        )?;

        let cxx_flags_str = cxx_flags.join(" ");

        build_system::BuildSystem::run_cmake_configure(
            name,
            &dep_source_dir,
            &build_dir,
            build_type,
            toolchain,
            &cxx_flags_str,
            &config.cmake_options,
            logger.clone(),
        )?;
        build_system::BuildSystem::run_cmake_build(name, &build_dir, build_type, logger.clone())?;

        let library_path =
            <build_system::builder::BuildSystem as build_system::ToolchainExecutor>::find_library_file(
                &build_dir,
                &lib_name,
                &config.output_type,
            )
            .ok_or_else(|| {
                anyhow!(
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
}
