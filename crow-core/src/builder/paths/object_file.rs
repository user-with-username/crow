use std::path::{Path, PathBuf};
use crate::builder::paths::SourceFilePath;

#[derive(Debug, Clone)]
pub struct ObjectFilePath(pub PathBuf);

impl ObjectFilePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn exists(&self) -> bool {
        self.0.exists()
    }
}
pub struct ObjectFileNaming;

impl ObjectFileNaming {
    pub fn generate(
        deps_dir: &Path,
        source: &SourceFilePath,
        hash: Option<&str>,
        release: bool,
    ) -> PathBuf {
        let stem = source.stem().unwrap_or_else(|| "unknown".to_string());

        let filename = if release {
            format!("{}.o", stem)
        } else {
            let short_hash = hash.map_or("nohash".to_string(), |h| h.chars().take(8).collect());
            format!("{}-{}.o", stem, short_hash)
        };

        deps_dir.join(filename)
    }
}
