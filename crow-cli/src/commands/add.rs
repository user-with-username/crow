use anyhowed::Result;
use clap::Args;
use crow_core::dependency::RegistryFetcher;
use crow_utils::status;
use std::fs;
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Value};

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
        let content = fs::read_to_string(&config_path)
            .map_err(|_| anyhowed::anyhow!("failed to read crow.toml"))?;

        let mut doc: DocumentMut = content
            .parse()
            .map_err(|e| anyhowed::anyhow!("failed to parse crow.toml: {}", e))?;

        if let Some(deps_table) = doc.get("dependencies").and_then(|t| t.as_table()) {
            if deps_table.contains_key(&self.args.package) {
                status!("Found", "{} already in dependencies", self.args.package);
                return Ok(());
            }
        }

        let deps_table = doc
            .entry("dependencies")
            .or_insert(Item::Table(toml_edit::Table::new()))
            .as_table_mut()
            .ok_or_else(|| anyhowed::anyhow!("expected table for dependencies"))?;

        deps_table.insert(
            &self.args.package,
            Item::Value(Value::String(toml_edit::Formatted::new(
                coords.version.clone(),
            ))),
        );

        fs::write(&config_path, doc.to_string())?;

        status!(
            "Adding",
            "{} v{} to dependencies",
            self.args.package,
            coords.version
        );
        Ok(())
    }
}
