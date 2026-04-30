
use crate::builder::incremental::{hash_files, BuildState};
use crate::project::Project;
use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

impl Project {
    fn compute_build_hash(&self) -> Result<String> {
        hash_files(&self.build_input_files())
    }

    pub fn should_build(&self, lock_hash: &str) -> Result<bool> {
        use crate::config::ProjectType;

        if matches!(self.package.r#type, ProjectType::HeaderOnly) {
            return Ok(false);
        }

        self.create_dirs()?;
        let state = BuildState::load_or_default(&self.project_state_path());
        let build_hash = self.compute_build_hash()?;
        let output_path = self.output_path();
        Ok(state.requires_rebuild(
            &build_hash,
            lock_hash,
            output_path.as_path().into(),
            self.profile.incremental(),
        ))
    }

    pub fn save_project_state(&self, lock_hash: &str) -> Result<()> {
        let state = BuildState {
            build_hash: self.compute_build_hash()?,
            lock_hash: lock_hash.to_string(),
            output: Some(self.output_path()),
        };
        state.save(&self.project_state_path())
    }

    fn build_input_files(&self) -> Vec<PathBuf> {
        let mut files = vec![self.manifest_path()];
        let mut seen = HashSet::new();

        for dir in self.config.build.src_dirs.iter().chain(self.config.build.include_dirs.iter()) {
            let root = if dir.is_relative() {
                self.root.join(dir)
            } else {
                dir.clone()
            };

            if !root.exists() {
                continue;
            }

            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.into_path();
                    if seen.insert(path.clone()) {
                        files.push(path);
                    }
                }
            }
        }

        files
    }

    fn project_state_path(&self) -> PathBuf {
        self.profile_dir()
            .join(format!(".{}.project-state.json", self.package.output_stem()))
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join("crow.toml")
    }
}
