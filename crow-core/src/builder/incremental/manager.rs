use anyhow::Result;
use crow_utils::normalize_path;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::builder::incremental::{CacheEntry, IncrementalCache};
use crate::builder::paths::{ObjectFilePath, SourceFilePath};

pub struct IncrementalManager {
    cache: Mutex<IncrementalCache>,
    path: PathBuf,
    is_incremental: bool,
}

impl IncrementalManager {
    pub fn new(cache_path: &Path, is_incremental: bool) -> Self {
        let cache = IncrementalCache::load_or_default(cache_path);
        Self {
            cache: Mutex::new(cache),
            path: cache_path.to_path_buf(),
            is_incremental,
        }
    }

    pub fn should_compile(
        &self,
        src: &SourceFilePath,
        hash: &Option<String>,
        obj: &ObjectFilePath,
    ) -> bool {
        if !self.is_incremental {
            return true;
        }

        let key = normalize_path(&src.as_path().to_string_lossy());
        
        let cache = self.cache.lock().unwrap();
        
        match (cache.files.get(&key), hash) {
            (None, _) | (_, None) => true,
            (Some(entry), Some(current_hash)) => {
                current_hash != &entry.hash
                    || !obj.exists()
                    || normalize_path(&obj.as_path().to_string_lossy())
                        != normalize_path(&entry.object.to_string_lossy())
            }
        }
    }

    pub fn record_success(&self, src: &SourceFilePath, entry: CacheEntry) {
        if !self.is_incremental {
            return;
        }

        let key = normalize_path(&src.as_path().to_string_lossy());
        
        let mut cache = self.cache.lock().unwrap();
        cache.files.insert(key, entry);
    }

    pub fn finalize(&self) -> Result<()> {
        if !self.is_incremental {
            let _ = std::fs::remove_file(&self.path);
            Ok(())
        } else {
            let cache = self.cache.lock().unwrap();
            cache.save(&self.path)
        }
    }
}