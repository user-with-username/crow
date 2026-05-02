use crate::builder::incremental::hash_files;
use crate::config::CrowConfig;
use crate::dependency::{DependencyResolver, ResolvedDependencyBuild, WheelArtifacts};
use crate::project::Project;
use anyhow::Result;
use crow_utils::status;
use crow_utils::progress::ProgressBar;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

enum Buildable {
    Project(Project),
    Wheel { name: String, root: PathBuf },
}

pub struct BuildSession<'a> {
    pub profile_name: &'a str,
    pub jobs: Option<usize>,
    pub progress: Option<ProgressBar>,
    pub built_count: usize,
    pub total_count: usize,
    pub total_duration: std::time::Duration,
}

impl<'a> BuildSession<'a> {
    pub fn new(profile_name: &'a str, jobs: Option<usize>) -> Self {
        Self {
            profile_name,
            jobs,
            progress: None,
            built_count: 0,
            total_count: 0,
            total_duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn build_root(
        mut self,
        config: CrowConfig,
        manifest_dir: PathBuf,
        resolved: ResolvedDependencyBuild,
    ) -> Result<Project> {
        let bootstrap = Project::new(config.clone(), manifest_dir.clone(), self.profile_name)?;
        let target_dir = bootstrap.target_dir();
        let cache_root = manifest_dir.join(&target_dir).join("dependency-cache");
        let resolver = DependencyResolver::new(cache_root);

        let mut compiler_flags = config.build.compiler.flags().to_vec();
        if let Some(pkg) = &config.package {
            if let Some(std) = &pkg.standard {
                if cfg!(target_os = "windows") {
                    compiler_flags.push(format!("/std:c++{}", std));
                } else {
                    compiler_flags.push(format!("-std=c++{}", std));
                }
            }
        }

        let mut buildable: Vec<(Buildable, String)> = Vec::new();
        let mut visited: HashSet<PathBuf> = HashSet::new();
        self.collect_buildable(
            &config,
            &manifest_dir,
            &resolved,
            &mut buildable,
            &mut visited,
        )?;

        self.total_count = buildable.len();

        if self.total_count > 0 {
            let first_label = buildable[0].1.clone();
            self.progress = Some(ProgressBar::new(self.total_count, first_label));
        }

        let mut wheel_artifacts: HashMap<PathBuf, WheelArtifacts> = HashMap::new();

        for (item, _label) in buildable {
            let start = Instant::now();

            match item {
                Buildable::Wheel { name, root } => {
                    let artifacts = self.compile_wheel(&resolver, &name, &root, &compiler_flags)?;
                    wheel_artifacts.insert(root, artifacts);
                }
                Buildable::Project(project) => {
                    let is_root = project.root == manifest_dir;
                    self.compile_project(&project, !is_root)?;
                }
            }

            self.total_duration += start.elapsed();
            self.built_count += 1;
            if let Some(pb) = &self.progress {
                pb.inc();
            }
        }

        let mut root_config = config.clone();
        Project::apply_dependency_standard(&mut root_config, &resolved);
        Project::merge_dependency_inputs(&mut root_config, &resolved);
        Self::apply_wheel_artifacts(&mut root_config, &wheel_artifacts);

        let root_project = Project::new(root_config, manifest_dir, self.profile_name)?;

        let opt_level = if root_project.profile.opt_level() != "0" {
            "optimized"
        } else {
            "unoptimized"
        };
        let debug_info = if root_project.profile.debug() {
            " + debuginfo"
        } else {
            ""
        };

        if let Some(pb) = &self.progress {
            pb.finish();
            println!(
                "\x1b[1;92m{:>12}\x1b[0m {} `{}` profile [{}{}] target(s) in {:.2}s",
                "Finished",
                root_project.package.name,
                self.profile_name,
                opt_level,
                debug_info,
                self.total_duration.as_secs_f32()
            );
        } else {
            status!(
                "Finished",
                "{} `{}` profile [{}{}] target(s)",
                root_project.package.name,
                self.profile_name,
                opt_level,
                debug_info
            );
        }

        Ok(root_project)
    }

    fn collect_buildable(
        &mut self,
        config: &CrowConfig,
        manifest_dir: &Path,
        resolved: &ResolvedDependencyBuild,
        out: &mut Vec<(Buildable, String)>,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        let canonical = Project::canonicalize_path(manifest_dir)?;
        if visited.contains(&canonical) {
            return Ok(());
        }
        visited.insert(canonical);

        for dep in &resolved.packages {
            if dep.is_wheel {
                let label = format!("{}(wheel)", dep.name);
                out.push((
                    Buildable::Wheel {
                        name: dep.name.clone(),
                        root: dep.root.clone(),
                    },
                    label,
                ));
                continue;
            }

            if matches!(
                dep.config.package.as_ref().map(|p| &p.r#type),
                Some(crate::config::ProjectType::HeaderOnly)
            ) {
                continue;
            }

            let dep_resolved = Project::resolve_dependencies(
                &dep.config,
                &dep.root,
                self.profile_name,
            )?;
            self.collect_buildable(&dep.config, &dep.root, &dep_resolved, out, visited)?;
        }

        let lockfile_path = manifest_dir.join("crow.lock");
        if !lockfile_path.exists() {
            let lockfile = crate::dependency::LockfileBuilder::build(config, resolved)?;
            lockfile.save(&lockfile_path)?;
        }

        let mut config_clone = config.clone();
        Project::apply_dependency_standard(&mut config_clone, resolved);
        Project::merge_dependency_inputs(&mut config_clone, resolved);

        let project = Project::new(config_clone, manifest_dir.to_path_buf(), self.profile_name)?;
        project.configure_parallelism(self.jobs);

        let lock_hash = hash_files(std::slice::from_ref(&lockfile_path))?;

        if project.should_build(&lock_hash)? {
            let label = format!("{} v{}", project.package.name, project.package.version);
            out.push((Buildable::Project(project), label));
        }

        Ok(())
    }

    fn compile_wheel(
        &self,
        resolver: &DependencyResolver,
        name: &str,
        root: &PathBuf,
        compiler_flags: &[String],
    ) -> Result<WheelArtifacts> {
        let display = format!("{} (wheel)", name);

        if let Some(pb) = &self.progress {
            pb.set_label(&display);
            pb.status("Compiling", &display);
        } else {
            status!("Compiling", "{}", display);
        }

        resolver.build_wheel(root, self.profile_name, compiler_flags)
    }

    fn compile_project(&self, project: &Project, is_dependency: bool) -> Result<()> {
        let lockfile_path = project.root.join("crow.lock");
        let lock_hash = hash_files(std::slice::from_ref(&lockfile_path))?;

        let display_path = if is_dependency {
            format!("{} v{}", project.package.name, project.package.version)
        } else {
            format!(
                "{} v{} ({})",
                project.package.name,
                project.package.version,
                project.root.display()
            )
        };

        if let Some(pb) = &self.progress {
            pb.set_label(&format!("{} v{}", project.package.name, project.package.version));
            pb.status("Compiling", &display_path);
        } else {
            status!("Compiling", "{}", display_path);
        }

        project.compile_and_link(&lock_hash)?;
        Ok(())
    }

    fn apply_wheel_artifacts(
        config: &mut CrowConfig,
        wheel_artifacts: &HashMap<PathBuf, WheelArtifacts>,
    ) {
        use std::collections::HashSet;

        let mut seen_include: HashSet<PathBuf> = config.build.include_dirs.iter().cloned().collect();
        let mut seen_libs: HashSet<String> = config.build.libs.iter().cloned().collect();
        let mut seen_lib_paths: HashSet<PathBuf> = config.build.lib_dirs.iter().cloned().collect();

        for artifacts in wheel_artifacts.values() {
            for include_dir in &artifacts.include_dirs {
                if seen_include.insert(include_dir.clone()) {
                    config.build.include_dirs.push(include_dir.clone());
                }
            }
            for lib_path in &artifacts.lib_paths {
                if seen_lib_paths.insert(lib_path.clone()) {
                    config.build.lib_dirs.push(lib_path.clone());
                }
            }
            for lib_name in &artifacts.lib_names {
                if seen_libs.insert(lib_name.clone()) {
                    config.build.libs.push(lib_name.clone());
                }
            }
        }
    }
}