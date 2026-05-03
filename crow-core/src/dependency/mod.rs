mod git;
mod graph;
mod lockfile;
mod merge;
mod wheel;

pub use git::GitDependencyFetcher;
pub use graph::DependencyGraph;
pub use lockfile::LockfileBuilder;
pub use merge::{apply_dependency_standard, format_lock_dependencies, merge_dependency_inputs};
pub use wheel::{create_wheel, WheelArtifacts, WheelType};

use crate::config::{BuildConfig, CrowConfig, LibraryConfig, Profiles};
use anyhow::{bail, Context, Result};
use crow_utils::normalize_path;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub root: PathBuf,
    pub config: CrowConfig,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub is_wheel: bool,
    pub build_flags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedDependencyBuild {
    pub packages: Vec<ResolvedPackage>,
    pub include_dirs: Vec<PathBuf>,
    pub libs: Vec<String>,
    pub lib_paths: Vec<PathBuf>,
    pub max_standard: Option<String>,
}

#[derive(Debug)]
pub struct DependencyResolver {
    cache_root: PathBuf,
    resolved_cache: HashMap<String, ResolvedPackage>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        let cache_root = dirs::home_dir()
            .expect("cannot find home directory")
            .join(".crow")
            .join("wheel_builds");
        std::fs::create_dir_all(&cache_root).expect("failed to create wheel builds cache");
        Self {
            cache_root,
            resolved_cache: HashMap::new(),
        }
    }

    pub fn build_wheel(
        &self,
        dep_root: &PathBuf,
        profile_name: &str,
        compiler_flags: &[String],
        build_flags: &[String],
    ) -> Result<WheelArtifacts> {
        let wheel_build_dir = self.cache_root.join("wheel_builds");
        let wheel = create_wheel(dep_root)
            .with_context(|| format!("no known build system found at {}", dep_root.display()))?;
        wheel.build(&wheel_build_dir, profile_name, compiler_flags, build_flags)
    }

    pub fn resolve_for(
        &mut self,
        root_config: &CrowConfig,
        root_dir: &Path,
        profile_name: &str,
        target_dir: &Path,
    ) -> Result<ResolvedDependencyBuild> {
        if root_config.dependencies.is_empty() {
            return Ok(ResolvedDependencyBuild::default());
        }

        std::fs::create_dir_all(&self.cache_root).with_context(|| {
            format!(
                "failed to create dependency cache at {}",
                self.cache_root.display()
            )
        })?;

        let root_dir = root_dir
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", root_dir.display()))?;
        let root_dir = PathBuf::from(normalize_path(&root_dir.display().to_string()));

        let mut graph = DependencyGraph::new();

        let root_idx = graph.add_node(
            root_dir.clone(),
            root_config.clone(),
            None,
            None,
            false,
            Vec::new(),
        )?;

        self.visit_dependencies(root_idx, &root_config.dependencies, &root_dir, &mut graph)?;

        let build_order = graph.resolve_order()?;

        let mut wheel_artifacts: HashMap<PathBuf, WheelArtifacts> = HashMap::new();
        let wheel_build_dir = self.cache_root.join("wheel_builds");

        let mut compiler_flags = root_config.build.compiler.flags().to_vec();

        if let Some(pkg) = &root_config.package {
            if let Some(std) = &pkg.standard {
                if cfg!(target_os = "windows") {
                    compiler_flags.push(format!("/std:c++{}", std));
                } else {
                    compiler_flags.push(format!("-std=c++{}", std));
                }
            }
        }

        for idx in &build_order {
            if *idx == root_idx {
                continue;
            }

            let payload = graph.get_node(*idx).context("failed to get node")?;
            if payload.is_wheel && !wheel_artifacts.contains_key(&payload.root) {
                if let Some(wheel) = create_wheel(&payload.root) {
                    match wheel.build(
                        &wheel_build_dir,
                        profile_name,
                        &compiler_flags,
                        &payload.build_flags,
                    ) {
                        Ok(artifacts) => {
                            wheel_artifacts.insert(payload.root.clone(), artifacts);
                        }
                        Err(e) => {
                            bail!(
                                "failed to build wheel for {}: {}",
                                payload.root.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        let mut resolved = ResolvedDependencyBuild::default();
        let mut seen_include = std::collections::HashSet::new();
        let mut seen_libs = std::collections::HashSet::new();
        let mut seen_lib_paths = std::collections::HashSet::new();
        let mut max_standard = crate::config::parse_standard(
            root_config
                .package
                .as_ref()
                .and_then(|pkg| pkg.standard.as_deref()),
        );

        for idx in build_order {
            if idx == root_idx {
                continue;
            }

            let payload = graph.get_node(idx).context("failed to get node")?;
            let package_name = payload
                .config
                .package
                .as_ref()
                .map(|pkg| pkg.name.clone())
                .unwrap_or_else(|| {
                    payload
                        .root
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });

            if payload.is_wheel {
                if let Some(artifacts) = wheel_artifacts.get(&payload.root) {
                    for include_dir in &artifacts.include_dirs {
                        if seen_include.insert(include_dir.clone()) {
                            resolved.include_dirs.push(include_dir.clone());
                        }
                    }
                    for lib_path in &artifacts.lib_paths {
                        if seen_lib_paths.insert(lib_path.clone()) {
                            resolved.lib_paths.push(lib_path.clone());
                        }
                    }
                    for lib_name in &artifacts.lib_names {
                        if seen_libs.insert(lib_name.clone()) {
                            resolved.libs.push(lib_name.clone());
                        }
                    }
                    resolved.packages.push(ResolvedPackage {
                        name: package_name,
                        root: payload.root.clone(),
                        config: payload.config.clone(),
                        source: payload.source.clone(),
                        checksum: payload.checksum.clone(),
                        is_wheel: true,
                        build_flags: payload.build_flags.clone(),
                    });
                }
                continue;
            }

            let package = payload
                .config
                .package
                .as_ref()
                .context("dependency manifest must have [package]")?;

            if package.r#type.is_bin() {
                bail!(
                    "dependency `{}` has unsupported package.type `{}`; only library-like dependencies are supported",
                    package.name,
                    package.r#type.type_str()
                );
            }

            resolved.packages.push(ResolvedPackage {
                name: package.name.clone(),
                root: payload.root.clone(),
                config: payload.config.clone(),
                source: payload.source.clone(),
                checksum: payload.checksum.clone(),
                is_wheel: false,
                build_flags: Vec::new(),
            });

            max_standard =
                max_standard.max(crate::config::parse_standard(package.standard.as_deref()));

            let include_dir = payload.root.join("include");
            if include_dir.exists() && seen_include.insert(include_dir.clone()) {
                resolved.include_dirs.push(include_dir);
            }

            if Self::is_linkable_type(&package.r#type) {
                let lib_path = package.output_path_in(&target_dir.join(profile_name));
                let link_item = lib_path.to_string_lossy().to_string();
                if seen_libs.insert(link_item.clone()) {
                    resolved.libs.push(link_item);
                }
            }
        }

        resolved.max_standard = max_standard.map(|s| s.to_string());

        Ok(resolved)
    }

    fn visit_dependencies(
        &mut self,
        owner_idx: petgraph::graph::NodeIndex,
        deps: &crate::config::Dependencies,
        owner_root: &Path,
        graph: &mut DependencyGraph,
    ) -> Result<()> {
        for (dep_name, spec) in deps.iter() {
            let resolved_dep = self.resolve_dependency(dep_name, spec, owner_root)?;
            let canonical_root = resolved_dep.root.clone();

            let dep_idx = graph.add_node(
                canonical_root.clone(),
                resolved_dep.config.clone(),
                resolved_dep.source.clone(),
                resolved_dep.checksum.clone(),
                resolved_dep.is_wheel,
                resolved_dep.build_flags.clone(),
            )?;

            graph.add_edge(owner_idx, dep_idx)?;

            if !graph.is_visiting(&canonical_root) {
                graph.mark_visiting(canonical_root.clone());
                self.visit_dependencies(
                    dep_idx,
                    &resolved_dep.config.dependencies,
                    &canonical_root,
                    graph,
                )?;
                graph.unmark_visiting(&canonical_root);
            }
        }
        Ok(())
    }

    fn resolve_dependency(
        &mut self,
        dep_name: &str,
        spec: &crate::config::DependencySpec,
        owner_root: &Path,
    ) -> Result<ResolvedPackage> {
        let source = spec.source();
        let source_build_flags = source.build_flags.clone();

        let cache_key = if let Some(git_url) = &source.git {
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
        } else if let Some(registry) = &source.registry {
            format!("registry:{}", registry)
        } else {
            dep_name.to_string()
        };

        if let Some(cached) = self.resolved_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let (dep_root, source_repr, checksum) = match (source.git, source.path, source.registry) {
            (Some(git_url), None, None) => {
                let (dep_root, rev) = GitDependencyFetcher::global().fetch(dep_name, &git_url)?;
                (dep_root, Some(format!("git+{}#{}", git_url, rev)), None)
            }
            (None, Some(path), None) => {
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
            (None, None, Some(_)) => {
                bail!("dependency `{dep_name}` uses `registry`, which is not implemented yet")
            }
            _ => bail!(
                "dependency `{dep_name}` has unsupported source; expected exactly one of git/path/registry"
            ),
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
                            r#type: crate::config::ProjectType::StaticLib(LibraryConfig::default()),
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
                } else {
                    anyhow::bail!(
                        "failed to load config for dependency `{dep_name}` at {} and no known build system found (CMakeLists.txt, meson.build, WORKSPACE)",
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

        self.resolved_cache.insert(cache_key, resolved_pkg.clone());

        Ok(resolved_pkg)
    }

    fn is_linkable_type(project_type: &crate::config::ProjectType) -> bool {
        project_type.is_static()
            || project_type.is_shared()
            || matches!(project_type, crate::config::ProjectType::Lib(_))
    }
}
