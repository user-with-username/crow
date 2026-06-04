use anyhowed::Result;
use clap::Args;
use crow_core::dependency::RegistryFetcher;
use crow_utils::{status, toml_editor::add_dep};
use std::path::PathBuf;

#[derive(Args)]
pub struct AddArgs {
    /// Package name to add
    pub package: String,

    /// Version requirement (if not specified, uses latest compatible version)
    #[arg(short, long)]
    pub version: Option<String>,

    /// Override registry URL
    #[arg(long)]
    pub registry: Option<String>,
}

pub struct AddCommand {
    args: AddArgs,
}

impl AddCommand {
    pub fn new(args: AddArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let registry_url = self
            .args
            .registry
            .as_deref()
            .unwrap_or(crow_utils::environment::DEFAULT_REGISTRY_URL);

        let version_req = self.args.version.as_deref().unwrap_or("*");

        let coords =
            RegistryFetcher::global().resolve_str(&self.args.package, version_req, registry_url)?;

        let config_path = PathBuf::from("crow.toml");

        add_dep(config_path, &self.args.package, &coords.version)?;

        status!(
            "Adding",
            "{} v{} to dependencies",
            self.args.package,
            coords.version
        );

        Ok(())
    }
}
