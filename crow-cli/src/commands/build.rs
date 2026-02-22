use anyhow::Result;
use clap::Args;
use crow_core::{
    builder::{CompilationBuilder, LinkingBuilder},
    CrowConfig, Project,
};
use crow_utils::status;
use rayon;
use std::time::Instant;

#[derive(Args)]
pub struct BuildArgs {
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,

    /// Build for the specific target triple
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
        let config = CrowConfig::load()?;
        let profile_name = if self.args.release {
            "release"
        } else {
            &self.args.profile
        };
        let project = Project::new(config, profile_name)?;

        if project.config.build.parallelism {
            if let Some(jobs) = self.args.jobs {
                let _ = rayon::ThreadPoolBuilder::new()
                    .num_threads(jobs)
                    .build_global();
            }
        }

        let compiler_exe = if let Some(path) = project.config.build.compiler.path() {
            path.as_str()
        } else {
            project.compiler_path()
        };

        let linker_exe = if let Some(path) = project.config.build.linker.path() {
            path.as_str()
        } else {
            project.linker_path()
        };

        status!(
            "Compiling",
            "{} v{} ({})",
            project.config.package.name,
            project.config.package.version,
            project.root.display(),
        );

        let start = Instant::now();
        let objects = CompilationBuilder::new(compiler_exe, &project).compile()?;
        LinkingBuilder::new(linker_exe, &project, &objects).link()?;

        let duration = start.elapsed();
        let opt_level = if project.profile.opt_level() != "0" {
            "optimized"
        } else {
            "unoptimized"
        };

        status!(
            "Finished",
            "`{}` profile [{}] target(s) in {:.2}s",
            profile_name,
            opt_level,
            duration.as_secs_f32()
        );

        Ok(())
    }
}
