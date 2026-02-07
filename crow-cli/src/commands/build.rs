use anyhow::Result;
use clap::Args;
use crow_core::{
    CrowConfig, Project, builder::{CompilationBuilder, LinkingBuilder}
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

        let compiler_exe = project.config.build.compiler.path
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                project.compiler_kind().default_executable()
            });

        let linker_kind = project.linker_kind();
        
        let linker_exe = if let Some(ref path) = project.config.build.linker.path {
            path.as_str()
        } else if linker_kind.is_msvc() {
            linker_kind.default_executable()
        } else {
            compiler_exe
        };

        status!(
            "Compiling",
            "{} v{} ({})",
            project.config.package.name,
            project.config.package.version,
            project.root.display()
        );

        let start = Instant::now();

        let objects = CompilationBuilder::new(compiler_exe, &project).compile()?;

        LinkingBuilder::new(linker_exe, &project, &objects).link()?;

        let duration = start.elapsed();
        let profile_name = if self.args.release { "release" } else { "dev" };
        let opt_level = if self.args.release {
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