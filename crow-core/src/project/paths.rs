use crate::project::Project;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

impl Project {
    pub fn find_sources(&self) -> Vec<PathBuf> {
        let extensions = &self.config.build.src_extensions;
        self.config
            .build
            .src_dirs
            .iter()
            .flat_map(|src_dir| {
                let dir = if src_dir.is_relative() {
                    self.root.join(src_dir)
                } else {
                    src_dir.clone()
                };
                WalkDir::new(dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|entry| {
                        entry
                            .path()
                            .extension()
                            .map_or(false, |ext| extensions.iter().any(|e| e.as_str() == ext))
                    })
                    .map(|entry| entry.path().to_path_buf())
                    .collect::<Vec<_>>()
            })
            .collect()
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
