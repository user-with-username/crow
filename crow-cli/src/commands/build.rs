use anyhow::Result;
use clap::Args;
use crow_core::{
    builder::{CompilationBuilder, LinkingBuilder},
    config::{ProjectType, Workspace},
    dependency::DependencyResolver,
    Project,
};
use crow_utils::status;
use rayon;
use std::{collections::HashSet, path::PathBuf, time::Instant};

#[derive(Args)]
pub struct BuildArgs {
    #[arg(short, long)]
    pub release: bool,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,
    #[arg(long)]
    pub bin: Option<String>,
    #[arg(short = 'p', long, default_value = "debug")]
    pub profile: String,
}

pub struct BuildCommand {
    args: BuildArgs,
}

impl BuildCommand {
    pub fn new(args: BuildArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let workspace = Workspace::load()?;

        let profile_name = if self.args.release {
            "release"
        } else {
            &self.args.profile
        };

        let members_to_build: Vec<_> = if let Some(bin_name) = &self.args.bin {
            workspace
                .binary_members()
                .into_iter()
                .filter(|(cfg, _)| cfg.package.as_ref().map(|p| &p.name) == Some(bin_name))
                .map(|&(ref cfg, ref root)| (cfg.clone(), root.clone()))
                .collect()
        } else {
            workspace
                .members()
                .map(|(cfg, root)| (cfg.clone(), root.clone()))
                .collect()
        };

        if members_to_build.is_empty() {
            if self.args.bin.is_some() {
                anyhow::bail!(
                    "No binary package named `{}` found",
                    self.args.bin.as_ref().unwrap()
                );
            } else {
                anyhow::bail!("No packages found to build");
            }
        }

        for (member_config, member_root) in members_to_build {
            self.build_package(member_config, member_root, profile_name)?;
        }

        Ok(())
    }

    fn build_package(
        &self,
        config: crow_core::CrowConfig,
        root: PathBuf,
        profile_name: &str,
    ) -> Result<()> {
        let project = Project::new(config.clone(), root.clone(), profile_name)?;

        let target_dir = project.target_dir();
        let resolver = DependencyResolver::new(root.join(&target_dir).join("dependency-cache"));

        let resolved = resolver.resolve_for(&config, &root, profile_name, &target_dir)?;

        for dependency in &resolved.packages {
            if matches!(
                dependency.config.package.as_ref().map(|p| &p.r#type),
                Some(ProjectType::HeaderOnly)
            ) {
                continue;
            }

            let mut dep_config = dependency.config.clone();
            if let Some(max_standard) = &resolved.max_standard {
                if let Some(dep_pkg) = dep_config.package.as_mut() {
                    dep_pkg.standard = Some(max_standard.clone());
                }
            }
            let dep_project = Project::new(dep_config, dependency.root.clone(), profile_name)?;
            self.compile_and_link(&dep_project, true)?;
        }

        let mut config_clone = config;
        Self::merge_dependency_inputs(&mut config_clone, &resolved);

        let project = Project::new(config_clone, root, profile_name)?;

        if project.config.build.parallelism {
            if let Some(jobs) = self.args.jobs {
                let _ = rayon::ThreadPoolBuilder::new()
                    .num_threads(jobs)
                    .build_global();
            }
        }

        let compiler_exe = project
            .config
            .build
            .compiler
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| project.compiler_path());

        let archiver_exe = project
            .config
            .build
            .archiver
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| project.archiver_path());

        let linker_exe = project
            .config
            .build
            .linker
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| project.linker_path());

        self.compile_and_link_with_tools(
            &project,
            compiler_exe,
            linker_exe,
            archiver_exe,
            profile_name,
            false,
        )
    }

    fn compile_and_link(&self, project: &Project, is_dependency: bool) -> Result<()> {
        let compiler_exe = project
            .config
            .build
            .compiler
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| project.compiler_path());

        let archiver_exe = project
            .config
            .build
            .archiver
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| project.archiver_path());

        let linker_exe = project
            .config
            .build
            .linker
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| project.linker_path());

        self.compile_and_link_with_tools(
            project,
            compiler_exe,
            linker_exe,
            archiver_exe,
            &project.profile_name,
            is_dependency,
        )
    }

    fn compile_and_link_with_tools(
        &self,
        project: &Project,
        compiler_exe: &str,
        linker_exe: &str,
        archiver_exe: &str,
        profile_name: &str,
        is_dependency: bool,
    ) -> Result<()> {
        status!(
            "Compiling",
            "{} v{} ({})",
            project.package.name,
            project.package.version,
            project.root.display()
        );

        let start = Instant::now();

        project.create_dirs()?;

        let objects = CompilationBuilder::new(compiler_exe, &project).compile()?;

        LinkingBuilder::new(linker_exe, archiver_exe, &project, &objects).link()?;

        let duration = start.elapsed();
        let opt_level = if project.profile.opt_level() != "0" {
            "optimized"
        } else {
            "unoptimized"
        };

        if !is_dependency {
            status!(
                "Finished",
                "{} `{}` profile [{}] target(s) in {:.2}s",
                project.package.name,
                profile_name,
                opt_level,
                duration.as_secs_f32()
            );
        }

        Ok(())
    }

    fn merge_dependency_inputs(
        config: &mut crow_core::CrowConfig,
        resolved: &crow_core::dependency::ResolvedDependencyBuild,
    ) {
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
}
