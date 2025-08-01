use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hasher;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BuildCache {
    pub entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub source_hash: u64,
    pub flags_hash: u64,
    #[serde(default)]
    pub deps_hash: u64,
    pub obj_path: String,
}

pub trait CacheManager {
    fn load_cache(cache_path: &Path, incremental: bool) -> anyhow::Result<BuildCache>;
    fn save_cache(cache_path: &Path, cache: &BuildCache) -> anyhow::Result<()>;
    fn parse_dep_file(dep_path: &Path) -> anyhow::Result<Vec<PathBuf>>;
    fn compute_deps_hash(deps: &[PathBuf]) -> anyhow::Result<u64>;
    fn compute_flags_hash(compiler: &str, args: &[std::ffi::OsString]) -> u64;
}

impl CacheManager for BuildCache {
    fn load_cache(cache_path: &Path, incremental: bool) -> anyhow::Result<BuildCache> {
        if incremental && cache_path.exists() {
            let content = std::fs::read_to_string(cache_path)?;
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            Ok(BuildCache::default())
        }
    }

    fn save_cache(cache_path: &Path, cache: &BuildCache) -> anyhow::Result<()> {
        std::fs::write(cache_path, serde_json::to_string_pretty(cache)?)?;
        Ok(())
    }

    fn parse_dep_file(dep_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let content = std::fs::read_to_string(dep_path)?;
        let normalized = content.replace("\\\r\n", " ").replace("\\\n", " ");
        let mut files = Vec::new();
        if let Some(pos) = normalized.find(':') {
            for part in normalized[pos + 1..].split_whitespace() {
                if !part.is_empty() {
                    files.push(PathBuf::from(part));
                }
            }
        }
        files.sort();
        files.dedup();
        Ok(files)
    }

    fn compute_deps_hash(deps: &[PathBuf]) -> anyhow::Result<u64> {
        let mut hashes = Vec::with_capacity(deps.len());
        for path in deps {
            if path.exists() {
                hashes.push(xxhash_rust::xxh3::xxh3_64(&std::fs::read(path)?));
            } else {
                hashes.push(0);
            }
        }
        hashes.sort();
        let mut hasher = xxhash_rust::xxh3::Xxh3::default();
        for hash in hashes {
            hasher.write_u64(hash);
        }
        Ok(hasher.finish())
    }

    fn compute_flags_hash(compiler: &str, args: &[std::ffi::OsString]) -> u64 {
        let mut hasher = xxhash_rust::xxh3::Xxh3::default();
        hasher.update(compiler.as_bytes());
        for arg in args {
            hasher.update(arg.to_string_lossy().as_bytes());
            hasher.update(&[0]);
        }
        hasher.finish()
    }
}
