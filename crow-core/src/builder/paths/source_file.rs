use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SourceFilePath(pub PathBuf);

impl SourceFilePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn stem(&self) -> Option<String> {
        self.0.file_stem().map(|s| s.to_string_lossy().into_owned())
    }
}
