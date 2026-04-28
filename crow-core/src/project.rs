use crate::builder::toolchain::{self, Toolchain};
use crate::builder::incremental::{hash_files, ProjectState};
use crate::builder::{CompilationBuilder, LinkingBuilder};
use crate::config::{CrowConfig, Dependencies, Package, ProjectType};
use crate::dependency::{DependencyResolver, ResolvedDependencyBuild, ResolvedPackage};
use crate::lockfile::{CrowLockfile, LockedPackage};
use anyhow::{Context, Result};
use crow_utils::enviroment::Enviroment;
use crow_utils::{normalize_path, status};
use rayon;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

pub struct Project {
    pub config: CrowConfig,
    pub package: Package,
    pub root: PathBuf,
    pub workspace_root: PathBuf,
    pub profile: crate::config::Profile,
    pub profile_name: String,
    pub toolchain: Box<dyn Toolchain>,
}

struct BuildSession<'a> {
    profile_name: &'a str,
    jobs: Option<usize>,
    built_roots: HashSet<PathBuf>,
}

impl<'a> BuildSession<'a> {
    fn new(profile_name: &'a str, jobs: Option<usize>) -> Self {
        Self {
            profile_name,
            jobs,
            built_roots: HashSet::new(),
        }
    }

    fn build_root(
        &mut self,
        config: CrowConfig,
        manifest_dir: PathBuf,
        resolved: ResolvedDependencyBuild,
    ) -> Result<Project> {
        self.build_project(config, manifest_dir, resolved, false)
    }

    fn build_project(
        &mut self,
        config: CrowConfig,
        manifest_dir: PathBuf,
        resolved: ResolvedDependencyBuild,
        is_dependency: bool,
    ) -> Result<Project> {
        let canonical_root = Project::canonicalize_path(&manifest_dir)?;
        let mut merged_config = config.clone();
        Project::apply_dependency_standard(&mut merged_config, &resolved);
        Project::merge_dependency_inputs(&mut merged_config, &resolved);
        let merged_project = Project::new(merged_config, manifest_dir.clone(), self.profile_name)?;

        if self.built_roots.contains(&canonical_root) {
            return Ok(merged_project);
        }

        for dependency in &resolved.packages {
            if matches!(
                dependency.config.package.as_ref().map(|p| &p.r#type),
                Some(ProjectType::HeaderOnly)
            ) {
                continue;
            }

            let dep_resolved = Project::resolve_dependencies(
                &dependency.config,
                &dependency.root,
                self.profile_name,
            )?;
            let _ = self.build_project(
                dependency.config.clone(),
                dependency.root.clone(),
                dep_resolved,
                true,
            )?;
        }

        let mut config = config;
        Project::apply_dependency_standard(&mut config, &resolved);
        let lockfile = Project::build_lockfile(&config, &resolved)?;
        let lockfile_path = manifest_dir.join("crow.lock");
        lockfile.save(&lockfile_path)?;

        Project::merge_dependency_inputs(&mut config, &resolved);
        let project = Project::new(config, manifest_dir, self.profile_name)?;
        project.configure_parallelism(self.jobs);

        self.built_roots.insert(canonical_root);
        let lock_hash = hash_files(std::slice::from_ref(&lockfile_path))?;

        let should_build = project.should_build(&lock_hash)?;
        let start = Instant::now();
        
        if should_build {
            if is_dependency {
                status!(
                    "Compiling",
                    "{} v{}",
                    project.package.name,
                    project.package.version
                );
            } else {
                status!(
                    "Compiling",
                    "{} v{} ({})",
                    project.package.name,
                    project.package.version,
                    project.root.display()
                );
            }
            project.compile_and_link(&lock_hash)?;
        }
        
        if !is_dependency {
            let duration = start.elapsed();
            let opt_level = if project.profile.opt_level() != "0" {
                "optimized"
            } else {
                "unoptimized"
            };
            
            let debug_info = if project.profile.debug() && project.profile.opt_level() == "0" {
                " + debuginfo"
            } else if project.profile.debug() {
                " + debuginfo"
            } else {
                ""
            };
            
            status!(
                "Finished",
                "{} `{}` profile [{}{}] target(s) in {:.2}s",
                project.package.name,
                self.profile_name,
                opt_level,
                debug_info,
                duration.as_secs_f32()
            );
        }

        Ok(project)
    }
}

impl Project {
    pub fn build(path: impl AsRef<Path>, profile_name: &str, jobs: Option<usize>) -> Result<Self> {
        let (config, manifest_dir) = CrowConfig::find_in_tree(path.as_ref())?;
        let resolved = Self::resolve_dependencies(&config, &manifest_dir, profile_name)?;
        let mut session = BuildSession::new(profile_name, jobs);
        session.build_root(config, manifest_dir, resolved)
    }

    pub fn new(
        loaded_config: CrowConfig,
        manifest_dir: PathBuf,
        profile_name: &str,
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
        })
    }

    pub fn find_sources(&self) -> Vec<PathBuf> {
        let extensions = &self.config.build.src_extensions;
        self.config
            .build
            .src_dirs
            .iter()
            .flat_map(|src_dir| {
                let dir = if src_dir.is_relative() {
                    self.root.join(src_dir)
                } else {
                    src_dir.clone()
                };
                WalkDir::new(dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|entry| {
                        entry
                            .path()
                            .extension()
                            .map_or(false, |ext| extensions.iter().any(|e| e.as_str() == ext))
                    })
                    .map(|entry| entry.path().to_path_buf())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn target_dir(&self) -> PathBuf {
        Enviroment::target_dir()
    }

    pub fn profile_dir(&self) -> PathBuf {
        self.target_dir().join(&self.profile_name)
    }

    pub fn build_dir(&self) -> PathBuf {
        self.profile_dir()
    }

    pub fn output_path(&self) -> PathBuf {
        self.package.output_path_in(&self.profile_dir())
    }

    pub fn output_name(&self) -> String {
        self.package.output_name()
    }

    pub fn create_dirs(&self) -> Result<(), std::io::Error> {
        let dirs = vec![
            self.target_dir(),
            self.profile_dir(),
            self.profile_dir().join("deps"),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
            }
        }

        Ok(())
    }

    pub fn clean(&self) -> Result<(), std::io::Error> {
        let target_dir = self.target_dir();
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        Ok(())
    }

    pub fn compiler_kind(&self) -> crate::builder::kinds::compiler_kind::CompilerKind {
        self.toolchain.compiler_kind()
    }

    pub fn linker_kind(&self) -> crate::builder::kinds::linker_kind::LinkerKind {
        self.toolchain.linker_kind()
    }

    pub fn compiler_path(&self) -> &str {
        self.toolchain.compiler_path()
    }

    pub fn linker_path(&self) -> &str {
        if let Some(path) = self.config.build.linker.path() {
            return path;
        }
        if self.compiler_kind().is_msvc() {
            self.toolchain.linker_path()
        } else {
            self.toolchain.compiler_path()
        }
    }

    pub fn archiver_path(&self) -> &str {
        self.toolchain.archiver_path()
    }

    pub fn archiver_kind(&self) -> crate::builder::kinds::archiver_kind::ArchiverKind {
        self.toolchain.archiver_kind()
    }

    pub fn system_include_dirs(&self) -> Vec<PathBuf> {
        self.toolchain.system_include_dirs()
    }

    fn configure_parallelism(&self, jobs: Option<usize>) {
        if self.config.build.parallelism {
            if let Some(jobs) = jobs {
                let _ = rayon::ThreadPoolBuilder::new()
                    .num_threads(jobs)
                    .build_global();
            }
        }
    }

    fn compile_and_link(&self, lock_hash: &str) -> Result<()> {
        let compiler_exe = self
            .config
            .build
            .compiler
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.compiler_path());

        let archiver_exe = self
            .config
            .build
            .archiver
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.archiver_path());

        let linker_exe = self
            .config
            .build
            .linker
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.linker_path());

        self.compile_and_link_with_tools(
            compiler_exe,
            linker_exe,
            archiver_exe,
            lock_hash,
        )
    }

    fn compile_and_link_with_tools(
        &self,
        compiler_exe: &str,
        linker_exe: &str,
        archiver_exe: &str,
        lock_hash: &str,
    ) -> Result<()> {
        self.create_dirs()?;

        let objects = CompilationBuilder::new(compiler_exe, self).compile()?;

        LinkingBuilder::new(linker_exe, archiver_exe, self, &objects).link()?;

        self.save_project_state(lock_hash)?;

        Ok(())
    }

    fn resolve_dependencies(
        config: &CrowConfig,
        manifest_dir: &Path,
        profile_name: &str,
    ) -> Result<ResolvedDependencyBuild> {
        let bootstrap_project = Self::new(config.clone(), manifest_dir.to_path_buf(), profile_name)?;
        let target_dir = bootstrap_project.target_dir();
        let resolver =
            DependencyResolver::new(manifest_dir.join(&target_dir).join("dependency-cache"));
        resolver.resolve_for(config, manifest_dir, profile_name, &target_dir)
    }

    fn build_lockfile(
        config: &CrowConfig,
        resolved: &ResolvedDependencyBuild,
    ) -> Result<CrowLockfile> {
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
                dependencies: Self::format_lock_dependencies(&config.dependencies, &by_name),
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
                dependencies: Self::format_lock_dependencies(
                    &dependency.config.dependencies,
                    &by_name,
                ),
            });
        }

        Ok(CrowLockfile::new(packages))
    }

    fn format_lock_dependencies(
        dependencies: &Dependencies,
        by_name: &HashMap<String, &ResolvedPackage>,
    ) -> Vec<String> {
        dependencies
            .iter()
            .map(|(dep_name, _)| {
                if let Some(dep) = by_name.get(dep_name) {
                    let mut entry = format!("{} {}", dep.name, dep.config.package
                        .as_ref()
                        .map(|pkg| pkg.version.as_str())
                        .unwrap_or("0.0.0"));
                    if let Some(source) = &dep.source {
                        entry.push_str(&format!(" ({source})"));
                    }
                    entry
                } else {
                    dep_name.clone()
                }
            })
            .collect()
    }

    fn project_state_path(&self) -> PathBuf {
        self.profile_dir()
            .join(format!(".{}.project-state.json", self.package.output_stem()))
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join("crow.toml")
    }

    fn build_input_files(&self) -> Vec<PathBuf> {
        let mut files = vec![self.manifest_path()];
        let mut seen = HashSet::new();

        for dir in self.config.build.src_dirs.iter().chain(self.config.build.include_dirs.iter()) {
            let root = if dir.is_relative() {
                self.root.join(dir)
            } else {
                dir.clone()
            };

            if !root.exists() {
                continue;
            }

            for entry in WalkDir::new(root).into_iter().filter_map(|entry| entry.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.into_path();
                    if seen.insert(path.clone()) {
                        files.push(path);
                    }
                }
            }
        }

        files
    }

    fn compute_build_hash(&self) -> Result<String> {
        hash_files(&self.build_input_files())
    }

    fn should_build(&self, lock_hash: &str) -> Result<bool> {
        if matches!(self.package.r#type, ProjectType::HeaderOnly) {
            return Ok(false);
        }

        self.create_dirs()?;
        let state = ProjectState::load_or_default(&self.project_state_path());
        let build_hash = self.compute_build_hash()?;
        let output_path = self.output_path();
        Ok(state.should_build(
            &build_hash,
            lock_hash,
            Some(output_path.as_path()),
            self.profile.incremental(),
        ))
    }

    fn save_project_state(&self, lock_hash: &str) -> Result<()> {
        let state = ProjectState {
            build_hash: self.compute_build_hash()?,
            lock_hash: lock_hash.to_string(),
            output: Some(self.output_path()),
        };
        state.save(&self.project_state_path())
    }

    fn canonicalize_path(path: &Path) -> Result<PathBuf> {
        let canonical = path
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", path.display()))?;
        Ok(PathBuf::from(normalize_path(&canonical.display().to_string())))
    }

    fn merge_dependency_inputs(config: &mut CrowConfig, resolved: &ResolvedDependencyBuild) {
        let mut include_seen: HashSet<_> = config.build.include_dirs.iter().cloned().collect();
        for include_dir in &resolved.include_dirs {
            if include_seen.insert(include_dir.clone()) {
                config.build.include_dirs.push(include_dir.clone());
            }
        }

        let mut libs_seen: HashSet<_> = config.build.libs.iter().cloned().collect();
        for lib in &resolved.libs {
            if libs_seen.insert(lib.clone()) {
                config.build.libs.push(lib.clone());
            }
        }
    }

    fn apply_dependency_standard(config: &mut CrowConfig, resolved: &ResolvedDependencyBuild) {
        if let Some(max_standard) = &resolved.max_standard {
            if let Some(package) = config.package.as_mut() {
                package.standard = Some(max_standard.clone());
            }
        }
    }
}