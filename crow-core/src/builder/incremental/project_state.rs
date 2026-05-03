use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildState {
    pub build_hash: String,
    pub lock_hash: String,
    pub output: Option<PathBuf>,
}

impl BuildState {
    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }

    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn requires_rebuild(
        &self,
        build_hash: &str,
        lock_hash: &str,
        output_path: Option<&Path>,
        is_incremental: bool,
    ) -> bool {
        if !is_incremental {
            return true;
        }

        if self.build_hash != build_hash || self.lock_hash != lock_hash {
            return true;
        }

        match (output_path, self.output.as_deref()) {
            (Some(output_path), Some(saved_output)) => {
                output_path != saved_output || !output_path.exists()
            }
            (Some(output_path), None) => !output_path.exists(),
            (None, _) => false,
        }
    }
}
