use anyhow::{Context, Result};
use crow_utils::normalize_path;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
            // Skip makefile rule lines (`target: prereqs`), but not Windows paths (`C:\...`).
            if line.contains(':') && !is_windows_drive_path(line) {
                continue;
            }

            let path = Path::new(line);
            if path.exists() {
                headers.push(path.to_path_buf());
            }
        }

        Ok(headers)
    }

    pub fn save_deps(dep_path: &Path, lines: &[String]) -> Result<()> {
        let mut deps = Vec::new();
        for line in lines {
            if let Some(path_str) = line.strip_prefix("Note: including file: ") {
                deps.push(normalize_path(path_str.trim()));
            }
        }
        if !deps.is_empty() {
            let mut f = fs::File::create(dep_path)?;
            for d in deps {
                writeln!(f, "{}", d)?;
            }
        }
        Ok(())
    }
}

fn is_windows_drive_path(line: &str) -> bool {
    let b = line.as_bytes();
    b.len() >= 3
        && b[0].is_ascii_alphabetic()
        && b[1] == b':'
        && (b[2] == b'\\' || b[2] == b'/')
}
