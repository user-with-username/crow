use crate::project::Project;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

impl Project {
    pub fn find_sources_in(&self, dirs: &[PathBuf]) -> Vec<PathBuf> {
        let extensions = &self.config.build.src_extensions;
        dirs.iter()
            .flat_map(|dir| {
                let root = if dir.is_relative() {
                    self.root.join(dir)
                } else {
                    dir.clone()
                };
                if !root.exists() {
                    return Vec::new();
                }
                WalkDir::new(root)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|entry| {
                        entry.file_type().is_file()
                            && entry
                                .path()
                                .extension()
                                .map_or(false, |ext| extensions.iter().any(|e| e.as_str() == ext))
                    })
                    .map(|entry| entry.path().to_path_buf())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn find_sources(&self) -> Vec<PathBuf> {
        self.find_sources_in(&self.config.build.src_dirs)
    }

    pub fn find_tests(&self) -> Vec<PathBuf> {
        self.find_sources_in(&self.config.build.test_dirs)
    }

    pub fn target_dir(&self) -> PathBuf {
        crow_utils::environment::Environment::target_dir()
    }

    pub fn profile_dir(&self) -> PathBuf {
        self.target_dir().join(&self.profile_name)
    }

    pub fn build_dir(&self) -> PathBuf {
        self.profile_dir()
    }

    pub fn output_path(&self) -> PathBuf {
        self.package.output_path_in(&self.profile_dir())
    }

    pub fn output_name(&self) -> String {
        self.package.output_name()
    }

    pub fn create_dirs(&self) -> Result<(), std::io::Error> {
        let dirs = vec![
            self.target_dir(),
            self.profile_dir(),
            self.profile_dir().join("deps"),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
            }
        }

        Ok(())
    }

    pub fn clean(&self) -> Result<(), std::io::Error> {
        let target_dir = self.target_dir();
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        Ok(())
    }

    pub(crate) fn canonicalize_path(path: &Path) -> Result<PathBuf> {
        let canonical = path
            .canonicalize()
            .with_context(|| format!("failed to resolve {}", path.display()))?;
        Ok(PathBuf::from(crow_utils::normalize_path(
            &canonical.display().to_string(),
        )))
    }
}
