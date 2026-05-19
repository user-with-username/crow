mod build;
mod build_session;
mod paths;
mod state;
mod test;

pub use build_session::BuildSession;

use crate::builder::toolchain::{self, Toolchain};
use crate::config::{CrowConfig, Package};
use crate::dependency::{DependencyResolver, ResolvedDependencyBuild};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Project {
    pub config: CrowConfig,
    pub package: Package,
    pub root: PathBuf,
    pub workspace_root: PathBuf,
    pub profile: crate::config::Profile,
    pub profile_name: String,
    pub toolchain: Box<dyn Toolchain>,
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

        let toolchain = toolchain::detect_toolchain(
            loaded_config.build.compiler.path().cloned(),
            Some(loaded_config.build.compiler.kind().clone()),
            loaded_config.build.linker.path().cloned(),
            Some(loaded_config.build.linker.kind().clone()),
            loaded_config.build.archiver.path().cloned(),
            Some(loaded_config.build.archiver.kind().clone()),
        )?;

        Ok(Self {
            config: loaded_config,
            package,
            workspace_root: manifest_dir.clone(),
            root: manifest_dir,
            profile,
            profile_name: profile_name.to_string(),
            toolchain,
            resolved_deps,
        })
    }

    pub fn build(path: impl AsRef<Path>, profile_name: &str, jobs: Option<usize>) -> Result<Self> {
        let (config, manifest_dir) = crate::config::CrowConfig::find_in_tree(path.as_ref())?;
        let resolved = Self::resolve_dependencies(&config, &manifest_dir, profile_name)?;
        let session = BuildSession::new(profile_name, jobs);
        session.build_root(config, manifest_dir, resolved)
    }

    fn resolve_dependencies(
        config: &CrowConfig,
        manifest_dir: &Path,
        profile_name: &str,
    ) -> Result<ResolvedDependencyBuild> {
        let bootstrap_project = Self::new(
            config.clone(),
            manifest_dir.to_path_buf(),
            profile_name,
            None,
        )?;
        let target_dir = bootstrap_project.target_dir();
        let compiler_path = bootstrap_project.compiler_path().to_string();
        let compiler_kind = bootstrap_project.compiler_kind();

        let mut resolver = DependencyResolver::new();
        resolver.resolve_for(
            config,
            manifest_dir,
            profile_name,
            &target_dir,
            Some(&compiler_path),
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
