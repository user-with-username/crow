use anyhow::Result;
use clap::Args;
use crow_core::dependency::RegistryFetcher;
use crow_utils::status;

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
            .unwrap_or(crow_core::dependency::DEFAULT_REGISTRY_URL);

        if let Some(ref ver) = self.args.version {
            status!("Deleting", "{} v{}", self.args.package, ver);
        } else {
            status!("Deleting", "{} (all versions)", self.args.package);
        }

        if self.args.dry_run {
            status!("Dry run", "would delete from {}", registry_url);
            status!("Dry run", "skipping actual deletion");
            return Ok(());
        }

        if !self.args.yes {
            if let Some(ref ver) = self.args.version {
                println!(
                    "Are you sure you want to delete {} v{} from registry?",
                    self.args.package, ver
                );
            } else {
                println!(
                    "Are you sure you want to delete ALL versions of {} from registry?",
                    self.args.package
                );
            }
            println!("This action cannot be undone!");
            print!("\nType 'yes' to confirm: ");

            use std::io::Write;
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim() != "yes" {
                status!("Cancelled", "deletion aborted");
                return Ok(());
            }
        }

        RegistryFetcher::delete(
            &self.args.package,
            self.args.version.as_deref(),
            registry_url,
        )?;

        if let Some(ref ver) = self.args.version {
            status!(
                "Success",
                "deleted {} v{} from {}",
                self.args.package,
                ver,
                registry_url
            );
        } else {
            status!(
                "Success",
                "deleted {} (all versions) from {}",
                self.args.package,
                registry_url
            );
        }

        Ok(())
    }
}