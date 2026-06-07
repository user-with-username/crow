use crate::project::Project;
use anyhowed::Result;
use std::path::PathBuf;
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

    pub fn find_benches(&self) -> Vec<PathBuf> {
        self.find_sources_in(&self.config.build.bench_dirs)
    }

    pub fn find_all(&self) -> Vec<PathBuf> {
        self.find_sources_in(&vec![
            std::env::current_dir().expect("Failed to get current dir")
        ])
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

    pub fn output_stem(&self) -> String {
        if let Some(target) = &self.active_target {
            return target.clone();
        }
        self.package.output_stem()
    }

    pub fn output_path(&self) -> PathBuf {
        self.profile_dir()
            .join(self.output_stem())
            .with_extension(self.package.r#type.extension())
    }

    pub fn output_name(&self) -> String {
        let stem = self.output_stem();
        let extension = self.package.r#type.extension();
        if extension.is_empty() {
            stem
        } else {
            format!("{}.{}", stem, extension)
        }
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
}
