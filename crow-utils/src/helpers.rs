use anyhow::{Context, Result};
use std::path::Path;

pub fn change_directory(path: &Path) -> Result<()> {
    std::env::set_current_dir(path)
        .context(format!("failed to change directory to {}", path.display()))
}
