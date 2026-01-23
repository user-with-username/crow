use anyhow::Result;
use clap::Args;
use crow_core::{
    builder::{CompilationBuilder, LinkingBuilder},
    compiler_kind::CompilerKind,
    CrowConfig, Project,
};
use crow_utils::status;
use std::time::Instant;

#[derive(Args)]
pub struct BuildArgs {
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,

    /// Target triple
    #[arg(long)]
    pub target: Option<String>,

    /// Number of parallel jobs
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Specific binary to build
    #[arg(long)]
    pub bin: Option<String>,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
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
        let project = Project::new(config, self.args.release)?;

        let compiler_kind = CompilerKind::detect(&project.config.build.compiler);
        let compiler_exe = project
            .config
            .build
            .compiler
            .clone()
            .unwrap_or_else(|| compiler_kind.default_executable().to_string());

        status!(
            "Compiling",
            "{} v{} ({})",
            project.config.package.name,
            project.config.package.version,
            project.root.display()
        );

        let start = Instant::now();

        let objects =
            CompilationBuilder::new(&compiler_exe, &project, self.args.verbose)
                .compile()?;

        LinkingBuilder::new(&compiler_exe, &project, &objects, self.args.verbose).link()?;

        let duration = start.elapsed();
        let profile_name = if self.args.release { "release" } else { "dev" };
        let opt_level = if self.args.release {
            "optimized"
        } else {
            "unoptimized"
        };

        status!(
            "Finished",
            "`{}` profile [{}] in {:.2}s",
            profile_name,
            opt_level,
            duration.as_secs_f32()
        );

        Ok(())
    }
}
