mod build;
mod build_session;
mod paths;
mod state;
mod runners;

pub use build_session::BuildSession;

use crate::builder::toolchain::{self, Toolchain};
use crate::config::{CrowConfig, Package};
use crate::dependency::{DependencyResolver, ResolvedDependencyBuild};
use anyhowed::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Project {
    pub config: CrowConfig,
    pub package: Package,
    pub root: PathBuf,
    pub workspace_root: PathBuf,
    pub profile: crate::config::Profile,
    pub profile_name: String,
    pub resolved_deps: Option<ResolvedDependencyBuild>,
}

impl Project {
    pub fn new(
        loaded_config: CrowConfig,
        manifest_dir: PathBuf,
        profile_name: &str,
        resolved_deps: Option<ResolvedDependencyBuild>,
    ) -> Result<Self> {
        let package = loaded_config
            .package
            .clone()
            .context("Manifest must have a [package] section")?;

        let profile = match profile_name {
            "dev" => crate::config::Profile::Dev(loaded_config.profile.dev.clone()),
            "release" => crate::config::Profile::Release(loaded_config.profile.release.clone()),
            "test" => crate::config::Profile::Test(loaded_config.profile.test.clone()),
            "bench" => crate::config::Profile::Bench(loaded_config.profile.bench.clone()),
            _ => crate::config::Profile::Dev(loaded_config.profile.dev.clone()),
        };

        Ok(Self {
            config: loaded_config,
            package,
            workspace_root: manifest_dir.clone(),
            root: manifest_dir,
            profile,
            profile_name: profile_name.to_string(),
            resolved_deps,
        })
    }

    pub(crate) fn toolchain(&self) -> Result<&dyn Toolchain> {
        toolchain::shared_toolchain(&self.config)
    }

    pub fn build(path: impl AsRef<Path>, profile_name: &str, jobs: Option<usize>) -> Result<Self> {
        let (config, manifest_dir) = crate::config::CrowConfig::find_in_tree(path.as_ref())?;
        let session = BuildSession::new(profile_name, jobs);
        session.build_root(config, manifest_dir)
    }

    pub(crate) fn resolve_dependencies(
        config: &CrowConfig,
        manifest_dir: &Path,
        profile_name: &str,
    ) -> Result<ResolvedDependencyBuild> {
        let target_dir = crow_utils::environment::Environment::target_dir();
        let compiler_kind = toolchain::compiler_kind_for_config(config)?;

        let mut resolver = DependencyResolver::new();
        resolver.resolve_for(
            config,
            manifest_dir,
            profile_name,
            &target_dir,
            config.build.compiler.path().map(|p| p.as_str()),
            compiler_kind,
        )
    }

    fn merge_dependency_inputs(config: &mut CrowConfig, resolved: &ResolvedDependencyBuild) {
        crate::dependency::merge_dependency_inputs(config, resolved);
    }

    fn apply_dependency_standard(config: &mut CrowConfig, resolved: &ResolvedDependencyBuild) {
        crate::dependency::apply_dependency_standard(config, resolved);
    }
}
