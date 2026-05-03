use crate::config::CrowConfig;
use crate::dependency::merge::format_lock_dependencies;
use crate::dependency::{ResolvedDependencyBuild, ResolvedPackage};
use crate::lockfile::{CrowLockfile, LockedPackage};
use anyhow::{Context, Result};
use std::collections::HashMap;

pub struct LockfileBuilder;

impl LockfileBuilder {
    pub fn build(config: &CrowConfig, resolved: &ResolvedDependencyBuild) -> Result<CrowLockfile> {
        let mut packages = Vec::new();
        let mut by_name: HashMap<String, &ResolvedPackage> = HashMap::new();

        for package in &resolved.packages {
            by_name.entry(package.name.clone()).or_insert(package);
        }

        if let Some(root_package) = config.package.as_ref() {
            packages.push(LockedPackage {
                name: root_package.name.clone(),
                version: root_package.version.clone(),
                source: None,
                checksum: None,
                dependencies: format_lock_dependencies(&config.dependencies, &by_name),
            });
        }

        for dependency in &resolved.packages {
            let package = dependency
                .config
                .package
                .as_ref()
                .context("dependency manifest must have [package]")?;

            packages.push(LockedPackage {
                name: package.name.clone(),
                version: package.version.clone(),
                source: dependency.source.clone(),
                checksum: dependency.checksum.clone(),
                dependencies: format_lock_dependencies(&dependency.config.dependencies, &by_name),
            });
        }

        Ok(CrowLockfile::new(packages))
    }
}
