use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PdbFilePath(pub PathBuf);

impl PdbFilePath {
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

    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        self.0.to_string_lossy()
    }
}

impl From<PathBuf> for PdbFilePath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<&Path> for PdbFilePath {
    fn from(path: &Path) -> Self {
        Self(path.to_path_buf())
    }
}

impl From<String> for PdbFilePath {
    fn from(s: String) -> Self {
        Self(PathBuf::from(s))
    }
}

impl From<&str> for PdbFilePath {
    fn from(s: &str) -> Self {
        Self(PathBuf::from(s))
    }
}

impl From<PdbFilePath> for PathBuf {
    fn from(path: PdbFilePath) -> Self {
        path.0
    }
}

pub struct PdbFileNaming;

impl PdbFileNaming {
    pub fn generate<P: AsRef<Path>, S: AsRef<str>>(directory: P, project_name: S) -> PdbFilePath {
        let filename = format!("{}.pdb", project_name.as_ref());
        PdbFilePath(directory.as_ref().join(filename))
    }
}
