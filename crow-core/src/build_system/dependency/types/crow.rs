use crate::build_system;
use crate::config::{Config, CrowDependencyBuild};
use crate::DependencyBuildOutput;
use anyhow::anyhow;
use crow_utils::logger::Logger;
use std::path::Path;

pub struct CrowDependency;

impl CrowDependency {
    pub fn build(
        name: &str,
        dep_source_path: &Path,
        crow_build_config: &CrowDependencyBuild,
        current_profile: &str,
        global_deps: bool,
        logger: Logger,
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
            build_system::BuildSystem::new(dep_config, current_profile, global_deps, logger)?;
        dep_build_system
            .build_internal(Some(1), Some(&dep_package_config))
            .map_err(|e| anyhow!("Failed to build Crow dependency '{}': {}", name, e))
    }
}
