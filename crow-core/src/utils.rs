use crate::config::PackageConfig;
use anyhow::Result;
use glob::glob;
use std::path::PathBuf;

pub fn find_source_files(package_config: &PackageConfig) -> Result<Vec<PathBuf>> {
    let mut sources: Vec<PathBuf> = Vec::new();
    for pattern in &package_config.sources {
        for entry in glob(pattern)? {
            sources.push(entry?);
        }
    }
    Ok(sources)
}
