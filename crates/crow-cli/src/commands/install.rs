use anyhowed::{Context, Result};
use clap::Args;
use crow_utils::environment::Environment;
use crow_utils::status;
use std::env;
use std::fs;
use std::path::PathBuf;

use super::run::build_project;

#[derive(Args)]
pub struct InstallArgs {
    /// Path to the package to install (like `cargo install --path`)
    #[arg(long, default_value = ".")]
    pub path: PathBuf,

    /// Install the binary into `<ROOT>/bin` instead of the default location
    #[arg(long)]
    pub root: Option<PathBuf>,

    /// Name of the specific binary package to install
    #[arg(long)]
    pub bin: Option<String>,

    /// Named target or conditional target (e.g., client, "cfg(windows)")
    #[arg(long)]
    pub target: Option<String>,

    /// Number of parallel jobs
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Build and install in debug mode (release is the default, like cargo)
    #[arg(long)]
    pub debug: bool,
}

pub struct InstallCommand {
    args: InstallArgs,
}

impl InstallCommand {
    pub fn new(args: InstallArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        // The workspace and the target directory are resolved relative to the
        // current directory, so build from inside the requested package.
        //
        // Do not `canonicalize()` the path: on Windows it yields a verbatim
        // `\\?\C:\...` path, which MSVC (`cl.exe`) cannot open. Letting the OS
        // resolve the path in `set_current_dir` keeps a plain `C:\...` form.
        env::set_current_dir(&self.args.path).with_context(|| {
            format!(
                "Cannot use `{}` as the package path",
                self.args.path.display()
            )
        })?;

        let profile = if self.args.debug { "debug" } else { "release" };

        let project = build_project(
            profile,
            self.args.jobs,
            self.args.bin,
            self.args.target.as_deref(),
        )?;

        let built = project.output_path();
        if !built.is_file() {
            anyhowed::bail!(
                "Executable '{}' not found. Did the build succeed?",
                built.display()
            );
        }

        let file_name = built
            .file_name()
            .with_context(|| format!("Invalid executable path `{}`", built.display()))?;

        let bin_dir = Environment::install_bin_dir(self.args.root.as_deref())?;
        fs::create_dir_all(&bin_dir)
            .with_context(|| format!("Failed to create `{}`", bin_dir.display()))?;

        let dest = bin_dir.join(file_name);

        status!("Installing", "`{}`", dest.display());

        fs::copy(&built, &dest).with_context(|| {
            format!(
                "Failed to copy `{}` to `{}` (is the binary currently running?)",
                built.display(),
                dest.display()
            )
        })?;

        status!("Installed", "package to `{}`", dest.display());

        if !Environment::is_in_path(&bin_dir) {
            eprintln!(
                "warning: `{}` is not in your PATH, add it to run the installed binary directly",
                bin_dir.display()
            );
        }

        Ok(())
    }
}
