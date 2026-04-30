use crate::config::parse_standard;
use crate::config::{CrowConfig, Dependencies, ProjectType};
use anyhow::{anyhow, bail, Context, Result};
use crow_utils::normalize_path;
use git2::{FetchOptions, RemoteCallbacks, Repository};
use petgraph::algo::toposort;
use petgraph::graph::{Graph, NodeIndex};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const CROW_MANIFEST: &str = "crow.toml";

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

#[derive(Debug, Clone)]
struct NodePayload {
    root: PathBuf,
    config: CrowConfig,
    source: Option<String>,
    checksum: Option<String>,
}

#[derive(Debug)]
pub struct DependencyResolver {
    cache_root: PathBuf,
}

impl DependencyResolver {
    pub fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
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

        let mut graph: Graph<NodePayload, ()> = Graph::new();
        let mut node_by_root: HashMap<PathBuf, NodeIndex> = HashMap::new();
        let mut root_seen = HashSet::new();

        let root_idx = self.add_node(
            &mut graph,
            &mut node_by_root,
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
            &mut node_by_root,
            &mut root_seen,
        )?;

        let sorted = toposort(&graph, None).map_err(|cycle| {
            let idx = cycle.node_id();
            let package_name = graph[idx]
                .config
                .package
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| graph[idx].root.display().to_string());
            anyhow!("dependency cycle detected near package `{package_name}`")
        })?;

        let mut build_order = sorted;
        build_order.reverse();

        let mut resolved = ResolvedDependencyBuild::default();
        let mut seen_include = HashSet::new();
        let mut seen_libs = HashSet::new();
        let mut max_standard = parse_standard(
            root_config
                .package
                .as_ref()
                .and_then(|pkg| pkg.standard.as_deref()),
        );

        for idx in build_order {
            if idx == root_idx {
                continue;
            }

            let payload = &graph[idx];
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
            max_standard = max_standard.max(parse_standard(package.standard.as_deref()));

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

    fn add_node(
        &self,
        graph: &mut Graph<NodePayload, ()>,
        node_by_root: &mut HashMap<PathBuf, NodeIndex>,
        root: PathBuf,
        config: CrowConfig,
        source: Option<String>,
        checksum: Option<String>,
    ) -> Result<NodeIndex> {
        if let Some(existing) = node_by_root.get(&root) {
            return Ok(*existing);
        }

        let idx = graph.add_node(NodePayload {
            root: root.clone(),
            config,
            source,
            checksum,
        });
        node_by_root.insert(root, idx);
        Ok(idx)
    }

    fn visit_dependencies(
        &self,
        owner_idx: NodeIndex,
        deps: &Dependencies,
        owner_root: &Path,
        graph: &mut Graph<NodePayload, ()>,
        node_by_root: &mut HashMap<PathBuf, NodeIndex>,
        visiting: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        for (dep_name, spec) in deps.iter() {
            let resolved_dep = self.resolve_dependency(dep_name, spec, owner_root)?;
            let canonical_root = resolved_dep.root.clone();

            let dep_idx = self.add_node(
                graph,
                node_by_root,
                canonical_root.clone(),
                resolved_dep.config.clone(),
                resolved_dep.source.clone(),
                resolved_dep.checksum.clone(),
            )?;
            if graph.find_edge(owner_idx, dep_idx).is_none() {
                graph.add_edge(owner_idx, dep_idx, ());
            }

            if visiting.insert(canonical_root.clone()) {
                self.visit_dependencies(
                    dep_idx,
                    &resolved_dep.config.dependencies,
                    &canonical_root,
                    graph,
                    node_by_root,
                    visiting,
                )?;
                visiting.remove(&canonical_root);
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
                let (dep_root, rev) = self.prepare_git_dependency(dep_name, &git_url)?;
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

        let manifest = self.find_manifest(&dep_root).with_context(|| {
            format!(
                "failed to locate `{CROW_MANIFEST}` for dependency `{dep_name}` at {}",
                dep_root.display()
            )
        })?;

        let manifest_dir = manifest
            .parent()
            .context("failed to get dependency manifest parent directory")?
            .to_path_buf();

        let dep_config = CrowConfig::load_from(&manifest_dir, true).map(|(config, _)| config)?;

        let canonical_root = manifest_dir.canonicalize().with_context(|| {
            format!(
                "failed to resolve dependency root {}",
                manifest_dir.display()
            )
        })?;
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

    fn prepare_git_dependency(&self, dep_name: &str, git_url: &str) -> Result<(PathBuf, String)> {
        use git2::build::CheckoutBuilder;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            git_url.hash(&mut hasher);
            hasher.finish()
        };
        let dep_dir = self.cache_root.join(format!("{}-{:016x}", dep_name, hash));

        let resolving_printed = Arc::new(AtomicBool::new(false));

        if dep_dir.join(".git").exists() {
            let repo = Repository::open(&dep_dir).with_context(|| {
                format!("failed to open git repository at {}", dep_dir.display())
            })?;

            let mut remote = repo
                .find_remote("origin")
                .context("failed to find remote 'origin'")?;

            let mut callbacks = RemoteCallbacks::new();
            let resolving_printed_clone = resolving_printed.clone();
            callbacks.transfer_progress(move |stats| {
                if stats.received_objects() == stats.total_objects() && stats.total_objects() > 0 {
                    if !resolving_printed_clone.swap(true, Ordering::Relaxed) {}
                } else {
                    resolving_printed_clone.store(false, Ordering::Relaxed);
                }
                true
            });

            let mut fetch_opts = FetchOptions::new();
            fetch_opts.remote_callbacks(callbacks);

            remote
                .fetch(
                    &["refs/heads/*:refs/remotes/origin/*"],
                    Some(&mut fetch_opts),
                    None,
                )
                .with_context(|| format!("failed to fetch from {git_url}"))?;

            eprintln!();

            let commit = repo
                .revparse_single("origin/HEAD")
                .or_else(|_| repo.revparse_single("origin/master"))
                .or_else(|_| repo.revparse_single("origin/main"))
                .with_context(|| format!("failed to find default branch for {git_url}"))?;

            let commit_id = commit.id();
            let obj = repo.find_object(commit_id, None)?;
            repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
                .with_context(|| format!("failed to checkout commit {commit_id}"))?;

            repo.set_head("refs/remotes/origin/HEAD")
                .or_else(|_| repo.set_head("refs/remotes/origin/master"))
                .or_else(|_| repo.set_head("refs/remotes/origin/main"))?;
        } else {
            // Clone new repository
            let mut callbacks = RemoteCallbacks::new();
            let resolving_printed_clone = resolving_printed.clone();
            callbacks.transfer_progress(move |stats| {
                if stats.received_objects() == stats.total_objects() && stats.total_objects() > 0 {
                    if !resolving_printed_clone.swap(true, Ordering::Relaxed) {
                        eprint!("Resolving deltas...");
                    }
                } else {
                    resolving_printed_clone.store(false, Ordering::Relaxed);
                }
                true
            });

            let mut fetch_opts = FetchOptions::new();
            fetch_opts.remote_callbacks(callbacks);

            let repo = Repository::init(&dep_dir).with_context(|| {
                format!("failed to initialize repository at {}", dep_dir.display())
            })?;

            let mut remote = repo
                .remote("origin", git_url)
                .with_context(|| format!("failed to add remote origin for {git_url}"))?;

            remote
                .fetch(
                    &["refs/heads/*:refs/remotes/origin/*"],
                    Some(&mut fetch_opts),
                    None,
                )
                .with_context(|| format!("failed to fetch from {git_url}"))?;

            eprintln!();

            let commit = repo
                .revparse_single("origin/HEAD")
                .or_else(|_| repo.revparse_single("origin/master"))
                .or_else(|_| repo.revparse_single("origin/main"))
                .with_context(|| format!("failed to find default branch for {git_url}"))?;

            let commit_id = commit.id();
            let obj = repo.find_object(commit_id, None)?;
            repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
                .with_context(|| format!("failed to checkout commit {commit_id}"))?;

            repo.set_head("refs/remotes/origin/HEAD")
                .or_else(|_| repo.set_head("refs/remotes/origin/master"))
                .or_else(|_| repo.set_head("refs/remotes/origin/main"))?;
        }

        let repo = Repository::open(&dep_dir)
            .with_context(|| format!("failed to open git repository at {}", dep_dir.display()))?;
        let head = repo.head().context("failed to read git dependency HEAD")?;
        let oid = head
            .target()
            .context("git dependency HEAD does not point to a direct commit")?;

        Ok((dep_dir, oid.to_string()))
    }

    fn find_manifest(&self, dep_root: &Path) -> Result<PathBuf> {
        let direct = dep_root.join(CROW_MANIFEST);
        if direct.exists() {
            return Ok(direct);
        }

        for entry in WalkDir::new(dep_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.file_name() == CROW_MANIFEST {
                return Ok(entry.into_path());
            }
        }

        bail!("no `{CROW_MANIFEST}` found under {}", dep_root.display())
    }

    fn is_linkable_type(project_type: &ProjectType) -> bool {
        project_type.is_static()
            || project_type.is_shared()
            || matches!(project_type, ProjectType::Lib(_))
    }
}
