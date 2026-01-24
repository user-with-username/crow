use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct CacheEntry {
    pub hash: String,
    pub object: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
pub struct IncrementalCache {
    pub files: HashMap<String, CacheEntry>,
}

impl IncrementalCache {
    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_else(|_e| Self::default())
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
