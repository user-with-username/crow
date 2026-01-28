use crate::builder::paths::SourceFilePath;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DependencyFilePath(pub PathBuf);

impl DependencyFilePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn exists(&self) -> bool {
        self.0.exists()
    }

    pub fn parent(&self) -> Option<&Path> {
        self.0.parent()
    }

    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name().and_then(|name| name.to_str())
    }

    pub fn file_stem(&self) -> Option<&str> {
        self.0.file_stem().and_then(|stem| stem.to_str())
    }

    pub fn extension(&self) -> Option<&str> {
        self.0.extension().and_then(|ext| ext.to_str())
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl From<PathBuf> for DependencyFilePath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<&Path> for DependencyFilePath {
    fn from(path: &Path) -> Self {
        Self(path.to_path_buf())
    }
}

impl From<String> for DependencyFilePath {
    fn from(s: String) -> Self {
        Self(PathBuf::from(s))
    }
}

impl From<&str> for DependencyFilePath {
    fn from(s: &str) -> Self {
        Self(PathBuf::from(s))
    }
}

impl From<DependencyFilePath> for PathBuf {
    fn from(path: DependencyFilePath) -> Self {
        path.0
    }
}

pub struct DependencyFileNaming;

impl DependencyFileNaming {
    pub fn generate(deps_dir: &Path, source: &SourceFilePath) -> DependencyFilePath {
        let stem = source.stem().unwrap_or_else(|| "unknown".to_string());
        let filename = format!("{}.d", stem);
        DependencyFilePath(deps_dir.join(filename))
    }
}
