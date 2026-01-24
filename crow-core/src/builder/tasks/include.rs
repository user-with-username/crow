use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug)]
pub struct IncludeTask {
    dep_file: PathBuf,
}

impl IncludeTask {
    pub fn new(dep_file: PathBuf) -> Self {
        Self { dep_file }
    }

    pub fn collect_headers(&self) -> Result<Vec<PathBuf>> {
        let content = fs::read_to_string(&self.dep_file)
            .with_context(|| format!("Failed to read {:?}", self.dep_file))?;

        let mut headers = Vec::new();

        for line in content.lines() {
            let line = line.trim_end_matches('\\').trim();
            if line.contains(':') {
                continue;
            }

            let path = Path::new(line);
            if path.exists() {
                headers.push(path.to_path_buf());
            }
        }

        Ok(headers)
    }
}
