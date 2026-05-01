use crate::builder::incremental::hash_files;
use crate::config::CrowConfig;
use crate::dependency::ResolvedDependencyBuild;
use crate::project::Project;
use anyhow::Result;
use crow_utils::status;
use crow_utils::progress::ProgressBar;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
        let mut buildable = Vec::new();
        let mut visited = std::collections::HashSet::new();
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

        let mut root_project = None;
        for (project, _) in buildable {
            let is_root = project.root == manifest_dir;
            let start = Instant::now();
            self.compile_project(&project, !is_root)?;
            let duration = start.elapsed();
            self.total_duration += duration;

            self.built_count += 1;
            if let Some(pb) = &self.progress {
                pb.inc();
            }
            if is_root {
                root_project = Some(project);
            }
        }

        let root_project = match root_project {
            Some(p) => p,
            None => Project::new(config, manifest_dir, self.profile_name)?,
        };

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
        out: &mut Vec<(Project, String)>,
        visited: &mut std::collections::HashSet<PathBuf>,
    ) -> Result<()> {
        let canonical = Project::canonicalize_path(manifest_dir)?;
        if visited.contains(&canonical) {
            return Ok(());
        }
        visited.insert(canonical);

        for dep in &resolved.packages {
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
            out.push((project, label));
        }

        Ok(())
    }

    fn compile_project(&self, project: &Project, is_dependency: bool) -> Result<()> {
        let lockfile_path = project.root.join("crow.lock");
        let lock_hash = hash_files(std::slice::from_ref(&lockfile_path))?;

        if let Some(pb) = &self.progress {
            pb.set_label(&format!("{} v{}", project.package.name, project.package.version));
            
            let display_path = if is_dependency {
                format!(
                    "{} v{}",
                    project.package.name,
                    project.package.version
                )
            } else {
                format!(
                    "{} v{} ({})",
                    project.package.name,
                    project.package.version,
                    project.root.display()
                )
            };
            
            pb.status("Compiling", &display_path);
        } else {
            let display_path = if is_dependency {
                format!(
                    "{} v{}",
                    project.package.name,
                    project.package.version
                )
            } else {
                format!(
                    "{} v{} ({})",
                    project.package.name,
                    project.package.version,
                    project.root.display()
                )
            };
            
            status!("Compiling", "{}", display_path);
        }

        project.compile_and_link(&lock_hash)?;
        Ok(())
    }
}