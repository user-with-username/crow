use anyhowed::{Context, Result};
use clap::Args;
use crow_core::{config::CrowConfig, dependency::RegistryFetcher};
use crow_utils::status;

#[derive(Args)]
pub struct PublishArgs {
    /// Override registry URL
    #[arg(long)]
    pub registry: Option<String>,

    /// Perform all checks without actually publishing
    #[arg(long)]
    pub dry_run: bool,

    /// Version to publish (defaults to package.version from crow.toml)
    #[arg(long)]
    pub version: Option<String>,
}

pub struct PublishCommand {
    args: PublishArgs,
}

impl PublishCommand {
    pub fn new(args: PublishArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let (config, manifest_dir) = CrowConfig::find_in_tree(&current_dir)?;

        let package = config
            .package
            .as_ref()
            .context("No [package] section found in crow.toml")?;

        let version = match &self.args.version {
            Some(v) => v.clone(),
            None => package.version.clone(),
        };

        if version != package.version {
            anyhowed::bail!(
                "Version mismatch: specified version `{}` does not match package version `{}`",
                version,
                package.version
            );
        }

        status!("Publishing", "{} v{}", package.name, version);

        if self.args.dry_run {
            status!("Dry run", "skipping actual publish");
            return Ok(());
        }

        let registry_url = self
            .args
            .registry
            .as_deref()
            .unwrap_or(crow_utils::environment::DEFAULT_REGISTRY_URL);

        let published =
            RegistryFetcher::publish(&manifest_dir, &package.name, &version, registry_url)?;

        if published {
            status!(
                "Success",
                "published {} v{} to {}",
                package.name,
                version,
                registry_url
            );
        } else {
            status!(
                "Up-to-date",
                "{} v{} already published",
                package.name,
                version
            );
        }

        Ok(())
    }
}
