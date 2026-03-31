use anyhow::Result;
use clap::Args;
use crow_core::{
    builder::{CompilationBuilder, LinkingBuilder},
    config::Workspace,
    Project,
};
use crow_utils::status;
use rayon;
use std::{path::PathBuf, time::Instant};

#[derive(Args)]
pub struct BuildArgs {
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,

    /// Build for a specific target triple
    #[arg(long)]
    pub target: Option<String>,

    /// Number of parallel jobs to run
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Build only the specified binary
    #[arg(long)]
    pub bin: Option<String>,

    /// Build profile (debug, release, test, bench)
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

        // Filter members by --bin if requested
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

        let is_multiple = members_to_build.len() > 1;

        for (member_config, member_root) in members_to_build {
            if is_multiple {
                status!("Building", "package at {}", member_root.display());
            }
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
        let project = Project::new(config, root, profile_name)?;

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

        status!(
            "Finished",
            "{} `{}` profile [{}] target(s) in {:.2}s",
            project.package.name,
            profile_name,
            opt_level,
            duration.as_secs_f32()
        );

        Ok(())
    }
}
