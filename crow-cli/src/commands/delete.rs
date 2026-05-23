use anyhowed::Result;
use clap::Args;
use crow_core::dependency::RegistryFetcher;
use crow_utils::status;
use dialoguer::console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};

#[derive(Args)]
pub struct DeleteArgs {
    /// Package name to delete
    pub package: String,

    /// Version to delete (if not specified, deletes all versions)
    #[arg(long)]
    pub version: Option<String>,

    /// Override registry URL
    #[arg(long)]
    pub registry: Option<String>,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Perform all checks without actually deleting
    #[arg(long)]
    pub dry_run: bool,
}

pub struct DeleteCommand {
    args: DeleteArgs,
}

impl DeleteCommand {
    pub fn new(args: DeleteArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let registry_url = self
            .args
            .registry
            .as_deref()
            .unwrap_or(crow_utils::environment::DEFAULT_REGISTRY_URL);

        let target = match &self.args.version {
            Some(version) => format!("{} v{}", self.args.package, version),
            None => format!("all versions of `{}`", self.args.package),
        };

        status!("Deleting", "{}", target);

        if self.args.dry_run {
            status!("Dry run", "would delete from {}", registry_url);
            status!("Dry run", "skipping actual deletion");
            return Ok(());
        }

        if !self.args.yes {
            println!("{}", style("This action cannot be undone.").red().bold());

            let confirmed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Delete {}?", target))
                .default(false)
                .wait_for_newline(false)
                .interact_opt()?;

            print!("\x1B[2A\x1B[2K\r\x1B[2K\r");

            match confirmed {
                Some(true) => {}
                _ => {
                    status!("Cancelled", "deletion aborted");
                    return Ok(());
                }
            }
        }

        RegistryFetcher::delete(
            &self.args.package,
            self.args.version.as_deref(),
            registry_url,
        )?;

        status!("Success", "deleted {} from {}", target, registry_url);

        Ok(())
    }
}
