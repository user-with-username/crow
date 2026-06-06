use crate::config::{
    BuildConfig, CrowConfig, DependencySpec, LibraryConfig, Profiles, ProjectType,
};
use crate::dependency::gitty::git::GitDependencyFetcher;
use crate::dependency::registry::RegistryFetcher;
use crate::dependency::wheel::create_wheel;
use crate::dependency::DependencyResolver;
use crate::dependency::ResolvedPackage;
use anyhowed::{bail, Context, Result};
use crow_utils::environment::{Environment, DEFAULT_REGISTRY_URL};
use crow_utils::normalize_path;
pub use semver::VersionReq;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

impl DependencyResolver {
    pub(crate) fn resolve_dependency(
        &mut self,
        dep_name: &str,
        spec: &DependencySpec,
        owner_root: &Path,
        profile_name: &str,
    ) -> Result<ResolvedPackage> {
        let build_flags = spec.build_flags().to_vec();

        let cache_key = match spec {
            DependencySpec::Git { git, .. } => format!("git:{git}"),
            DependencySpec::Path { path, .. } => {
                let abs_path = if path.is_relative() {
                    owner_root.join(path)
                } else {
                    path.clone()
                };
                format!("path:{}", abs_path.canonicalize().unwrap_or(abs_path).display())
            }
            DependencySpec::Registry { version, registry, .. } => {
                let registry_url = registry.as_deref().unwrap_or(&DEFAULT_REGISTRY_URL);
                format!("registry:{registry_url}:{dep_name}:{version}")
            }
            DependencySpec::Version(version) => {
                format!("registry:{DEFAULT_REGISTRY_URL}:{dep_name}:{version}")
            }
            DependencySpec::System { .. } => dep_name.to_string(),
        };

        if let Some(profile_cache) = self.resolved_cache.get(profile_name) {
            if let Some(cached) = profile_cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let (dep_root, source_repr, checksum) = match spec {
            DependencySpec::Git { git, .. } => {
                let (dep_root, rev) = GitDependencyFetcher::global().fetch(dep_name, git)?;
                (dep_root, Some(format!("git+{git}#{rev}")), None)
            }
            DependencySpec::Path { path, .. } => {
                let candidate = if path.is_relative() {
                    owner_root.join(path)
                } else {
                    path.clone()
                };
                let canonical = candidate.canonicalize().with_context(|| {
                    format!("failed to resolve path dependency `{dep_name}` from {}", owner_root.display())
                })?;
                
                let path_str = canonical.to_string_lossy();
                (PathBuf::from(normalize_path(&path_str)), None, None)
            }
            DependencySpec::Registry { version, registry, .. } => {
                let registry_url = registry.as_deref().unwrap_or(&DEFAULT_REGISTRY_URL);
                self.resolve_from_registry_optimized(dep_name, version, registry_url, &build_flags)?
            }
            DependencySpec::Version(version) => {
                let registry_url = Environment::registry_url();
                self.resolve_from_registry_optimized(dep_name, version, &registry_url, &build_flags)?
            }
            DependencySpec::System { .. } => {
                bail!("system dependency should not reach resolve_dependency");
            }
        };

        let load_result = CrowConfig::load_from(&dep_root, true);
        let (dep_config, is_wheel) = match load_result {
            Ok((config, _)) => (config, false),
            Err(_) => {
                let (r#type, is_wheel) = if create_wheel(&dep_root).is_some() {
                    (ProjectType::StaticLib(LibraryConfig::default()), true)
                } else if Self::has_include_dir(&dep_root) {
                    (ProjectType::HeaderOnly, false)
                } else {
                    anyhowed::bail!(
                        "failed to load config for dependency `{dep_name}` at {} \
                         and no known build system found (CMakeLists.txt, meson.build, WORKSPACE)",
                        dep_root.display()
                    )
                };

                let dummy_config = CrowConfig {
                    package: Some(crate::config::Package {
                        name: dep_name.to_string(),
                        version: "0.0.0".to_string(),
                        r#type,
                        standard: None,
                        authors: None,
                        description: None,
                        license: None,
                        repository: None,
                    }),
                    workspace: None,
                    build: BuildConfig::default(),
                    dependencies: crate::config::Dependencies::default(),
                    profile: Profiles::default(),
                };
                (dummy_config, is_wheel)
            }
        };

        let canonical_root = dep_root
            .canonicalize()
            .with_context(|| format!("failed to resolve dependency root {}", dep_root.display()))?;
        let canonical_root = PathBuf::from(normalize_path(&canonical_root.to_string_lossy()));

        let package_name = dep_config
            .package
            .as_ref()
            .map(|pkg| pkg.name.clone())
            .unwrap_or_else(|| dep_name.to_string());

        let resolved_pkg = ResolvedPackage {
            name: package_name,
            root: canonical_root,
            config: dep_config,
            source: source_repr,
            checksum,
            is_wheel,
            build_flags,
        };

        let profile_cache = self
            .resolved_cache
            .entry(profile_name.to_string())
            .or_insert_with(HashMap::new);
        
        profile_cache.insert(cache_key, resolved_pkg.clone());

        Ok(resolved_pkg)
    }

    fn has_include_dir(root: &Path) -> bool {
        ["include", "single_include", "src", "inc", "headers"]
            .iter()
            .any(|dir| root.join(dir).is_dir())
    }

    fn resolve_from_registry_optimized(
        &self,
        dep_name: &str,
        version_req: &VersionReq,
        registry_url: &str,
        _build_flags: &[String],
    ) -> Result<(PathBuf, Option<String>, Option<String>)> {
        let coords = RegistryFetcher::global().resolve(dep_name, version_req, registry_url)?;

        let (dep_root, resolved_commit) = GitDependencyFetcher::global().fetch_commit(
            dep_name,
            &coords.git_url,
            &coords.commit,
        )?;

        let source_repr = Some(format!(
            "registry+{registry_url}#{dep_name}@{}",
            &resolved_commit[..8.min(resolved_commit.len())]
        ));
        let checksum = Some(resolved_commit);

        Ok((dep_root, source_repr, checksum))
    }
}