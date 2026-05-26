use crate::builder::kinds::compiler_kind::CompilerKind;
use crate::config::{parse_standard, CrowConfig};
use crate::dependency::core::constraint::DependencyConstraint;
use crate::dependency::graph::DependencyGraph;
use crate::dependency::wheel::{create_wheel, load_wheel_cache, mark_wheel_built, WheelArtifacts};
use crate::dependency::{ResolvedDependencyBuild, ResolvedPackage};
use anyhowed::{Context, Result};
use crow_utils::normalize_path;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct DependencyResolver {
    cache_root: PathBuf,
    pub resolved_cache: HashMap<String, HashMap<String, ResolvedPackage>>,
    dependency_constraints: HashMap<String, HashMap<String, DependencyConstraint>>,
    dependency_owners: HashMap<String, HashMap<String, Vec<String>>>,
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
            dependency_constraints: HashMap::new(),
            dependency_owners: HashMap::new(),
        }
    }

    pub fn wheel_cache_dir(
        &self,
        dep_root: &Path,
        profile_name: &str,
        compiler_flags: &[String],
        build_flags: &[String],
        compiler_kind: CompilerKind,
    ) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        dep_root.hash(&mut hasher);
        compiler_kind.hash(&mut hasher);
        compiler_flags.hash(&mut hasher);
        build_flags.hash(&mut hasher);
        let hash = format!("{:016x}", hasher.finish());
        self.cache_root.join(profile_name).join(hash)
    }

    pub fn build_wheel(
        &self,
        dep_root: &PathBuf,
        profile_name: &str,
        compiler_flags: &[String],
        build_flags: &[String],
        compiler_path: Option<&str>,
        compiler_kind: CompilerKind,
    ) -> Result<WheelArtifacts> {
        let unique_build_dir = self.wheel_cache_dir(
            dep_root,
            profile_name,
            compiler_flags,
            build_flags,
            compiler_kind,
        );
        std::fs::create_dir_all(&unique_build_dir)?;

        if let Some(artifacts) = load_wheel_cache(&unique_build_dir, dep_root, profile_name) {
            return Ok(artifacts);
        }

        let wheel = create_wheel(dep_root)
            .with_context(|| format!("no known build system found at {}", dep_root.display()))?;
        let artifacts = wheel.build(
            &unique_build_dir,
            profile_name,
            compiler_flags,
            build_flags,
            compiler_path,
            compiler_kind,
        )?;
        mark_wheel_built(&unique_build_dir)?;
        Ok(artifacts)
    }

    pub(crate) fn check_dependency_conflict(
        &mut self,
        profile_name: &str,
        dep_name: &str,
        constraint: DependencyConstraint,
        owner_name: &str,
    ) -> Result<()> {
        let profile_constraints = self
            .dependency_constraints
            .entry(profile_name.to_string())
            .or_insert_with(HashMap::new);
        let profile_owners = self
            .dependency_owners
            .entry(profile_name.to_string())
            .or_insert_with(HashMap::new);

        if let Some(existing) = profile_constraints.get(dep_name) {
            if existing.conflicts_with(&constraint) {
                let owners = profile_owners
                    .get(dep_name)
                    .map(|v| v.join(", "))
                    .unwrap_or_else(|| "unknown".to_string());

                anyhowed::bail!(
                    "conflict for `{}`: {} requires {}, but {} requires {}\n",
                    dep_name,
                    owner_name,
                    constraint.display_short(),
                    owners,
                    existing.display_short()
                );
            }
        } else {
            profile_constraints.insert(dep_name.to_string(), constraint);
            profile_owners
                .entry(dep_name.to_string())
                .or_insert_with(Vec::new)
                .push(owner_name.to_string());
        }
        Ok(())
    }

    pub fn resolve_for(
        &mut self,
        root_config: &CrowConfig,
        root_dir: &Path,
        profile_name: &str,
        target_dir: &Path,
        _compiler_path: Option<&str>,
        compiler_kind: CompilerKind,
    ) -> Result<ResolvedDependencyBuild> {
        self.dependency_constraints.remove(profile_name);
        self.dependency_owners.remove(profile_name);

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
        let root_package_name = root_config
            .package
            .as_ref()
            .map(|pkg| pkg.name.as_str())
            .unwrap_or("root");

        let root_idx = graph.add_node(
            root_dir.clone(),
            root_config.clone(),
            None,
            None,
            false,
            Vec::new(),
        )?;

        self.visit_dependencies(
            root_idx,
            &root_config.dependencies,
            &root_dir,
            &mut graph,
            profile_name,
            root_package_name,
        )?;

        let build_order = graph.resolve_order()?;
        let mut wheel_artifacts: HashMap<PathBuf, WheelArtifacts> = HashMap::new();

        let compiler_flags = root_config.build.compiler.flags().to_vec();

        for idx in &build_order {
            if *idx == root_idx {
                continue;
            }

            let payload = graph.get_node(*idx).context("failed to get node")?;
            if payload.is_wheel && !wheel_artifacts.contains_key(&payload.root) {
                let cache_dir = self.wheel_cache_dir(
                    &payload.root,
                    profile_name,
                    &compiler_flags,
                    &payload.build_flags,
                    compiler_kind,
                );
                if let Some(artifacts) = load_wheel_cache(&cache_dir, &payload.root, profile_name) {
                    wheel_artifacts.insert(payload.root.clone(), artifacts);
                }
            }
        }

        let mut resolved = ResolvedDependencyBuild::default();
        let mut seen_include = HashSet::new();
        let mut seen_libs = HashSet::new();
        let mut seen_system_libs = HashSet::new();
        let mut max_standard = parse_standard(
            root_config
                .package
                .as_ref()
                .and_then(|pkg| pkg.standard.as_deref()),
        );

        for (_dep_name, spec) in root_config.dependencies.iter() {
            if spec.is_system() {
                for lib in spec.system_libs() {
                    if seen_system_libs.insert(lib.clone()) {
                        resolved.system_libs.push(lib);
                    }
                }
            }
        }

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
                    resolved.absorb_wheel_artifacts(artifacts);
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
                continue;
            }

            let package = match payload.config.package.as_ref() {
                Some(p) => p,
                None => continue,
            };

            resolved.packages.push(ResolvedPackage {
                name: package_name,
                root: payload.root.clone(),
                config: payload.config.clone(),
                source: payload.source.clone(),
                checksum: payload.checksum.clone(),
                is_wheel: false,
                build_flags: Vec::new(),
            });

            max_standard = max_standard.max(parse_standard(package.standard.as_deref()));

            let include_dir = payload.root.join("include");
            if include_dir.exists() && seen_include.insert(include_dir.clone()) {
                resolved.include_dirs.push(include_dir);
            }

            let lib_path = package.output_path_in(&target_dir.join(profile_name));
            if Self::is_linkable_type(&package.r#type) {
                let link_item = lib_path.to_string_lossy().to_string();
                if seen_libs.insert(link_item.clone()) {
                    resolved.libs.push(link_item);
                }
            }
        }

        resolved.max_standard = max_standard.map(|s| s.to_string());
        Ok(resolved)
    }

    fn is_linkable_type(project_type: &crate::config::ProjectType) -> bool {
        use crate::config::ProjectType;
        project_type.is_static()
            || project_type.is_shared()
            || matches!(project_type, ProjectType::Lib(_))
    }
}
