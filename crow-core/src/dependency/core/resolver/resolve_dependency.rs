use crate::config::{
    BuildConfig, CrowConfig, DependencySpec, LibraryConfig, Profiles, ProjectType,
};
use crate::dependency::git::GitDependencyFetcher;
use crate::dependency::registry::RegistryFetcher;
use crate::dependency::version_req::VersionReq;
use crate::dependency::wheel::create_wheel;
use crate::dependency::DependencyResolver;
use crate::dependency::ResolvedPackage;
use anyhowed::{bail, Context, Result};
use crow_utils::environment::{Environment, DEFAULT_REGISTRY_URL};
use crow_utils::normalize_path;
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
        let source = spec.source();
        let source_build_flags = source
            .as_ref()
            .map(|s| s.build_flags.clone())
            .unwrap_or_default();

        let version_req_str = spec.version_req();

        let cache_key = if let Some(source) = &source {
            if let Some(git_url) = &source.git {
                format!("git:{}", git_url)
            } else if let Some(path) = &source.path {
                let abs_path = if path.is_relative() {
                    owner_root.join(path)
                } else {
                    path.clone()
                };
                format!(
                    "path:{}",
                    abs_path.canonicalize().unwrap_or(abs_path).display()
                )
            } else if let Some(registry_url) = &source.registry {
                let version_req = version_req_str.as_deref().unwrap_or("*");
                format!("registry:{}:{}:{}", registry_url, dep_name, version_req)
            } else if source.version.is_some() {
                let version_req = version_req_str.as_deref().unwrap_or("*");
                format!(
                    "registry:{}:{}:{}",
                    DEFAULT_REGISTRY_URL, dep_name, version_req
                )
            } else {
                dep_name.to_string()
            }
        } else {
            dep_name.to_string()
        };

        if let Some(profile_cache) = self.resolved_cache.get(profile_name) {
            if let Some(cached) = profile_cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let (dep_root, source_repr, checksum) = if let Some(source) = &source {
            match (
                source.git.clone(),
                source.path.clone(),
                source.registry.clone(),
                source.version.clone(),
            ) {
                (Some(git_url), None, None, _) => {
                    let (dep_root, rev) =
                        GitDependencyFetcher::global().fetch(dep_name, &git_url)?;
                    (dep_root, Some(format!("git+{}#{}", git_url, rev)), None)
                }
                (None, Some(path), None, _) => {
                    let candidate = if path.is_relative() {
                        owner_root.join(path)
                    } else {
                        path
                    };
                    let canonical = candidate.canonicalize().with_context(|| {
                        format!(
                            "failed to resolve path dependency `{dep_name}` from {}",
                            owner_root.display()
                        )
                    })?;
                    (
                        PathBuf::from(normalize_path(&canonical.display().to_string())),
                        None,
                        None,
                    )
                }
                (None, None, Some(registry_url), Some(version_req)) => self.resolve_from_registry(
                    dep_name,
                    &version_req,
                    &registry_url,
                    &source_build_flags,
                )?,
                (None, None, None, Some(version_req)) => {
                    let registry_url = Environment::registry_url();
                    self.resolve_from_registry(
                        dep_name,
                        &version_req,
                        &registry_url,
                        &source_build_flags,
                    )?
                }
                _ => bail!(
                    "dependency `{dep_name}` has unsupported source; expected exactly one of \
                     git / path / version (with optional registry)"
                ),
            }
        } else {
            bail!("system dependency should not reach resolve_dependency");
        };

        let load_result = CrowConfig::load_from(&dep_root, true);
        let (dep_config, is_wheel) = match load_result {
            Ok((config, _)) => (config, false),
            Err(_) => {
                if create_wheel(&dep_root).is_some() {
                    let dummy_config = CrowConfig {
                        package: Some(crate::config::Package {
                            name: dep_name.to_string(),
                            version: "0.0.0".to_string(),
                            r#type: ProjectType::StaticLib(LibraryConfig::default()),
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
                    (dummy_config, true)
                } else if Self::has_include_dir(&dep_root) {
                    let dummy_config = CrowConfig {
                        package: Some(crate::config::Package {
                            name: dep_name.to_string(),
                            version: "0.0.0".to_string(),
                            r#type: ProjectType::HeaderOnly,
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
                    (dummy_config, false)
                } else {
                    anyhowed::bail!(
                        "failed to load config for dependency `{dep_name}` at {} \
                         and no known build system found (CMakeLists.txt, meson.build, WORKSPACE)",
                        dep_root.display()
                    )
                }
            }
        };

        let canonical_root = dep_root
            .canonicalize()
            .with_context(|| format!("failed to resolve dependency root {}", dep_root.display()))?;
        let canonical_root = PathBuf::from(normalize_path(&canonical_root.display().to_string()));

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
            build_flags: source_build_flags,
        };

        let profile_cache = self
            .resolved_cache
            .entry(profile_name.to_string())
            .or_insert_with(HashMap::new);
        profile_cache.insert(cache_key, resolved_pkg.clone());

        Ok(resolved_pkg)
    }

    /// Check whether the directory has a conventional header-only layout
    /// (include/, single_include/, src/, inc/ or headers/ subdirectory exists).
    fn has_include_dir(root: &Path) -> bool {
        ["include", "single_include", "src", "inc", "headers"]
            .iter()
            .any(|dir| root.join(dir).is_dir())
    }

    fn resolve_from_registry(
        &self,
        dep_name: &str,
        version_req: &str,
        registry_url: &str,
        _build_flags: &[String],
    ) -> Result<(PathBuf, Option<String>, Option<String>)> {
        let version_req = VersionReq::parse(version_req)?;

        let coords = RegistryFetcher::global().resolve(dep_name, &version_req, registry_url)?;

        let (dep_root, resolved_commit) = GitDependencyFetcher::global().fetch_commit(
            dep_name,
            &coords.git_url,
            &coords.commit,
        )?;

        let source_repr = Some(format!(
            "registry+{}#{}@{}",
            registry_url,
            dep_name,
            &resolved_commit[..8.min(resolved_commit.len())]
        ));
        let checksum = Some(resolved_commit);

        Ok((dep_root, source_repr, checksum))
    }
}
