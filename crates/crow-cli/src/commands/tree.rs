use anyhowed::Result;
use clap::Args;
use crow_core::config::Workspace;
use crow_core::dependency::DependencyResolver;

/// Displays the project's dependency tree.
///
/// Mirrors `cargo tree` behavior, showing dependencies in a hierarchical format
/// with Unicode box-drawing characters and duplicate detection.
#[derive(Args)]
pub struct TreeArgs {
    #[arg(short, long)]
    pub release: bool,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(short = 'p', long, default_value = "debug")]
    pub profile: String,
    /// Maximum display depth of the dependency tree.
    #[arg(short = 'D', long)]
    pub depth: Option<u32>,
    /// Do not de-duplicate; repeated dependencies are shown in full.
    #[arg(long)]
    pub no_dedupe: bool,
}

/// The `crow tree` command implementation.
///
/// Resolves the project's dependency graph via `DependencyResolver`
/// and prints it using `DependencyGraph::print_tree()`.
pub struct TreeCommand {
    args: TreeArgs,
}

impl TreeCommand {
    pub fn new(args: TreeArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let workspace = Workspace::load_with_target(self.args.target.as_deref())?;
        let profile_name = if self.args.release {
            "release"
        } else {
            &self.args.profile
        };

        for (_member_config, member_root) in workspace.members() {
            let config = _member_config.clone();
            let target_dir = crow_utils::environment::Environment::target_dir();
            let compiler_kind = crow_core::builder::toolchain::compiler_kind_for_config(&config)?;
            let mut resolver = DependencyResolver::new();
            let _resolved = resolver.resolve_for(
                &config,
                member_root.as_path(),
                profile_name,
                &target_dir,
                config.build.compiler.path().map(|p| p.as_str()),
                compiler_kind,
            )?;

            if let Some(graph) = resolver.graph() {
                graph.print_tree(
                    self.args.depth.unwrap_or(u32::MAX) as usize,
                    self.args.no_dedupe,
                );
            }
        }
        Ok(())
    }
}
