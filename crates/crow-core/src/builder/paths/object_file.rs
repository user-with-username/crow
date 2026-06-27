use crate::builder::{kinds::compiler_kind::CompilerKind, paths::SourceFilePath};
use std::path::{Path, PathBuf};

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
        compiler_kind: CompilerKind,
    ) -> PathBuf {
        let stem = source.stem().unwrap_or_else(|| "unknown".to_string());

        let extension = if compiler_kind.is_msvc() { "obj" } else { "o" };

        let filename = format!("{}.{}", stem, extension);

        deps_dir.join(filename)
    }
}
