mod graph;
mod git;
mod merge;
mod lockfile;

pub use graph::DependencyGraph;
pub use git::GitDependencyFetcher;
pub use merge::{merge_dependency_inputs, apply_dependency_standard, format_lock_dependencies};
pub use lockfile::{LockfileBuilder};

use crate::config::CrowConfig;
use anyhow::{bail, Context, Result};
use crow_utils::normalize_path;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub root: PathBuf,
    pub config: CrowConfig,
    pub source: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedDependencyBuild {
    pub packages: Vec<ResolvedPackage>,
    pub include_dirs: Vec<PathBuf>,
    pub libs: Vec<String>,
    pub max_standard: Option<String>,
}

#[derive(Debug)]
pub struct DependencyResolver {
    cache_root: PathBuf,
    git_fetcher: GitDependencyFetcher,
}

impl DependencyResolver {
    pub fn new(cache_root: PathBuf) -> Self {
        Self {
            cache_root: cache_root.clone(),
            git_fetcher: GitDependencyFetcher::new(cache_root),
        }
    }

    pub fn resolve_for(
        &self,
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
        )?;

        self.visit_dependencies(
            root_idx,
            &root_config.dependencies,
            &root_dir,
            &mut graph,
        )?;

        let build_order = graph.resolve_order()?;

        let mut resolved = ResolvedDependencyBuild::default();
        let mut seen_include = std::collections::HashSet::new();
        let mut seen_libs = std::collections::HashSet::new();
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
            });

            max_standard = max_standard.max(crate::config::parse_standard(package.standard.as_deref()));

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
        &self,
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
        &self,
        dep_name: &str,
        spec: &crate::config::DependencySpec,
        owner_root: &Path,
    ) -> Result<ResolvedPackage> {
        let source = spec.source();
        let (dep_root, source_repr, checksum) = match (source.git, source.path, source.registry) {
            (Some(git_url), None, None) => {
                let (dep_root, rev) = self.git_fetcher.fetch(dep_name, &git_url)?;
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

        let (dep_config, _) = CrowConfig::load_from(&dep_root, true)
            .with_context(|| format!("failed to load config for dependency `{dep_name}` at {}", dep_root.display()))?;

        let canonical_root = dep_root.canonicalize()
            .with_context(|| format!("failed to resolve dependency root {}", dep_root.display()))?;
        let canonical_root = PathBuf::from(normalize_path(&canonical_root.display().to_string()));

        let package_name = dep_config
            .package
            .as_ref()
            .map(|pkg| pkg.name.clone())
            .unwrap_or_else(|| dep_name.to_string());

        Ok(ResolvedPackage {
            name: package_name,
            root: canonical_root,
            config: dep_config,
            source: source_repr,
            checksum,
        })
    }

    fn is_linkable_type(project_type: &crate::config::ProjectType) -> bool {
        project_type.is_static()
            || project_type.is_shared()
            || matches!(project_type, crate::config::ProjectType::Lib(_))
    }
}