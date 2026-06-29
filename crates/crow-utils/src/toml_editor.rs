use anyhowed::Result;
use std::fs;
use std::path::PathBuf;
use toml_edit::{Document, Item, Value};

use crate::status;

pub fn add_dep(config_path: PathBuf, dep: &str, version: &str, is_dev: bool) -> Result<()> {
    let current_dependency_type: &str = if !is_dev {"dependencies"} else { "dev-dependencies" };
    
    let content = fs::read_to_string(&config_path)
        .map_err(|_| anyhowed::anyhow!("failed to read crow.toml"))?;

    let mut doc: Document = content
        .parse()
        .map_err(|e| anyhowed::anyhow!("failed to parse crow.toml: {}", e))?;

    // Check if dependency already exists
    if let Some(deps_table) = doc.get(current_dependency_type).and_then(|t| t.as_table()) {
        if deps_table.contains_key(dep) {
            status!("Found", "{} already in dependencies", dep);
            return Ok(());
        }
    }

    // Add dependency to table
    let deps_table = doc
        .entry(current_dependency_type)
        .or_insert(Item::Table(toml_edit::Table::new()))
        .as_table_mut()
        .ok_or_else(|| anyhowed::anyhow!("expected table for dependencies"))?;

    deps_table.insert(
        dep,
        Item::Value(Value::String(toml_edit::Formatted::new(
            version.to_string(),
        ))),
    );

    fs::write(&config_path, doc.to_string())?;

    Ok(())
}
