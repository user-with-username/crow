#[cfg(test)]
mod tests {
    use crow_core::builder::incremental::CacheEntry;
    use crow_core::builder::incremental::IncrementalCache;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_cache_entry_new() {
        let entry = CacheEntry {
            hash: "abc123".to_string(),
            object: PathBuf::from("obj/main.o"),
        };

        assert_eq!(entry.hash, "abc123");
        assert_eq!(entry.object, PathBuf::from("obj/main.o"));
    }

    #[test]
    fn test_incremental_cache_default() {
        let cache = IncrementalCache::default();

        assert!(cache.files.is_empty());
        assert_eq!(cache.files.len(), 0);
    }

    #[test]
    fn test_incremental_cache_new() {
        let mut files = HashMap::new();
        files.insert(
            "src/main.cpp".to_string(),
            CacheEntry {
                hash: "def456".to_string(),
                object: PathBuf::from("obj/main.o"),
            },
        );

        let cache = IncrementalCache { files };

        assert_eq!(cache.files.len(), 1);
        assert!(cache.files.contains_key("src/main.cpp"));
    }

    #[test]
    fn test_load_or_default_with_nonexistent_file() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("nonexistent_cache.json");

        let cache = IncrementalCache::load_or_default(&cache_path);

        assert!(cache.files.is_empty());
    }

    #[test]
    fn test_load_or_default_with_existing_file() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let mut files = HashMap::new();
        files.insert(
            "test.cpp".to_string(),
            CacheEntry {
                hash: "test123".to_string(),
                object: PathBuf::from("test.o"),
            },
        );
        let cache = IncrementalCache { files };
        cache.save(&cache_path)?;

        let loaded_cache = IncrementalCache::load_or_default(&cache_path);

        assert_eq!(loaded_cache.files.len(), 1);
        assert!(loaded_cache.files.contains_key("test.cpp"));
        assert_eq!(loaded_cache.files["test.cpp"].hash, "test123");
        assert_eq!(
            loaded_cache.files["test.cpp"].object,
            PathBuf::from("test.o")
        );

        Ok(())
    }

    #[test]
    fn test_load_with_nonexistent_file() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("missing.json");

        let result = IncrementalCache::load(&cache_path);

        assert!(result.is_ok());
        let cache = result.unwrap();
        assert!(cache.files.is_empty());
    }

    #[test]
    fn test_load_with_existing_file() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let mut files = HashMap::new();
        files.insert(
            "file1.cpp".to_string(),
            CacheEntry {
                hash: "hash1".to_string(),
                object: PathBuf::from("file1.o"),
            },
        );
        files.insert(
            "file2.cpp".to_string(),
            CacheEntry {
                hash: "hash2".to_string(),
                object: PathBuf::from("file2.o"),
            },
        );
        let cache = IncrementalCache { files };
        cache.save(&cache_path)?;

        let loaded_cache = IncrementalCache::load(&cache_path)?;

        assert_eq!(loaded_cache.files.len(), 2);
        assert_eq!(loaded_cache.files["file1.cpp"].hash, "hash1");
        assert_eq!(loaded_cache.files["file2.cpp"].hash, "hash2");

        Ok(())
    }

    #[test]
    fn test_load_with_invalid_json() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("invalid.json");

        fs::write(&cache_path, "this is not valid json")?;

        let result = IncrementalCache::load(&cache_path);

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_save_cache() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("saved_cache.json");

        let mut files = HashMap::new();
        files.insert(
            "source.cpp".to_string(),
            CacheEntry {
                hash: "abc".to_string(),
                object: PathBuf::from("source.o"),
            },
        );
        let cache = IncrementalCache { files };

        cache.save(&cache_path)?;

        assert!(cache_path.exists());

        let contents = fs::read_to_string(&cache_path)?;
        assert!(contents.contains("source.cpp"));
        assert!(contents.contains("abc"));
        assert!(contents.contains("source.o"));

        Ok(())
    }

    #[test]
    fn test_save_and_load_roundtrip() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("roundtrip.json");

        let mut original_files = HashMap::new();
        original_files.insert(
            "main.cpp".to_string(),
            CacheEntry {
                hash: "main_hash".to_string(),
                object: PathBuf::from("obj/main.o"),
            },
        );
        original_files.insert(
            "util.cpp".to_string(),
            CacheEntry {
                hash: "util_hash".to_string(),
                object: PathBuf::from("obj/util.o"),
            },
        );
        let original_cache = IncrementalCache {
            files: original_files,
        };

        original_cache.save(&cache_path)?;

        let loaded_cache = IncrementalCache::load(&cache_path)?;

        assert_eq!(original_cache.files.len(), loaded_cache.files.len());
        for (key, entry) in &original_cache.files {
            assert!(loaded_cache.files.contains_key(key));
            assert_eq!(loaded_cache.files[key].hash, entry.hash);
            assert_eq!(loaded_cache.files[key].object, entry.object);
        }

        Ok(())
    }

    #[test]
    fn test_empty_cache_save() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("empty.json");

        let empty_cache = IncrementalCache::default();
        empty_cache.save(&cache_path)?;

        assert!(cache_path.exists());

        let contents = fs::read_to_string(&cache_path)?;
        assert!(contents.contains("files"));
        assert!(contents.contains("{}"));

        Ok(())
    }

    #[test]
    fn test_load_empty_cache_file() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("empty_cache.json");

        let empty_cache = IncrementalCache::default();
        empty_cache.save(&cache_path)?;

        let loaded_cache = IncrementalCache::load(&cache_path)?;

        assert!(loaded_cache.files.is_empty());

        Ok(())
    }

    #[test]
    fn test_cache_with_special_characters_in_path() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let mut files = HashMap::new();
        files.insert(
            "path/with/spaces and/special chars.cpp".to_string(),
            CacheEntry {
                hash: "hash123".to_string(),
                object: PathBuf::from("output/path/with spaces.o"),
            },
        );
        let cache = IncrementalCache { files };

        cache.save(&cache_path)?;
        let loaded_cache = IncrementalCache::load(&cache_path)?;

        assert_eq!(loaded_cache.files.len(), 1);
        let key = "path/with/spaces and/special chars.cpp";
        assert!(loaded_cache.files.contains_key(key));
        assert_eq!(
            loaded_cache.files[key].object,
            PathBuf::from("output/path/with spaces.o")
        );

        Ok(())
    }

    #[test]
    fn test_cache_with_long_hash() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let long_hash = "a".repeat(64); // sha-256 successfully simulated
        let mut files = HashMap::new();
        files.insert(
            "large_file.cpp".to_string(),
            CacheEntry {
                hash: long_hash.clone(),
                object: PathBuf::from("large_file.o"),
            },
        );
        let cache = IncrementalCache { files };

        cache.save(&cache_path)?;
        let loaded_cache = IncrementalCache::load(&cache_path)?;

        assert_eq!(loaded_cache.files["large_file.cpp"].hash, long_hash);

        Ok(())
    }

    #[test]
    fn test_multiple_save_overwrite() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("overwrite.json");

        let mut files1 = HashMap::new();
        files1.insert(
            "file1.cpp".to_string(),
            CacheEntry {
                hash: "hash1".to_string(),
                object: PathBuf::from("file1.o"),
            },
        );
        let cache1 = IncrementalCache { files: files1 };
        cache1.save(&cache_path)?;

        let mut files2 = HashMap::new();
        files2.insert(
            "file2.cpp".to_string(),
            CacheEntry {
                hash: "hash2".to_string(),
                object: PathBuf::from("file2.o"),
            },
        );
        let cache2 = IncrementalCache { files: files2 };
        cache2.save(&cache_path)?;

        let loaded_cache = IncrementalCache::load(&cache_path)?;
        assert_eq!(loaded_cache.files.len(), 1);
        assert!(loaded_cache.files.contains_key("file2.cpp"));
        assert!(!loaded_cache.files.contains_key("file1.cpp"));

        Ok(())
    }

    #[test]
    fn test_cache_entry_serialization() -> anyhowed::Result<()> {
        let entry = CacheEntry {
            hash: "test_hash".to_string(),
            object: PathBuf::from("test.o"),
        };

        let serialized = serde_json::to_string(&entry)?;
        assert!(serialized.contains("test_hash"));
        assert!(serialized.contains("test.o"));

        let deserialized: CacheEntry = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.hash, entry.hash);
        assert_eq!(deserialized.object, entry.object);

        Ok(())
    }

    #[test]
    fn test_incremental_cache_serialization() -> anyhowed::Result<()> {
        let mut files = HashMap::new();
        files.insert(
            "test.cpp".to_string(),
            CacheEntry {
                hash: "abc".to_string(),
                object: PathBuf::from("test.o"),
            },
        );
        let cache = IncrementalCache { files };

        let serialized = serde_json::to_string_pretty(&cache)?;
        assert!(serialized.contains("test.cpp"));
        assert!(serialized.contains("abc"));
        assert!(serialized.contains("test.o"));

        let deserialized: IncrementalCache = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.files.len(), 1);
        assert_eq!(deserialized.files["test.cpp"].hash, "abc");

        Ok(())
    }

    #[test]
    fn test_cache_entry_clone() {
        let entry1 = CacheEntry {
            hash: "hash".to_string(),
            object: PathBuf::from("file.o"),
        };
        let entry2 = CacheEntry {
            hash: entry1.hash.clone(),
            object: entry1.object.clone(),
        };

        assert_eq!(entry1.hash, entry2.hash);
        assert_eq!(entry1.object, entry2.object);
    }

    #[test]
    fn test_update_cache_entry() {
        let mut cache = IncrementalCache::default();

        cache.files.insert(
            "source.cpp".to_string(),
            CacheEntry {
                hash: "old_hash".to_string(),
                object: PathBuf::from("source.o"),
            },
        );

        cache.files.insert(
            "source.cpp".to_string(),
            CacheEntry {
                hash: "new_hash".to_string(),
                object: PathBuf::from("source.o"),
            },
        );

        assert_eq!(cache.files.len(), 1);
        assert_eq!(cache.files["source.cpp"].hash, "new_hash");
    }

    #[test]
    fn test_remove_cache_entry() {
        let mut cache = IncrementalCache::default();

        cache.files.insert(
            "temp.cpp".to_string(),
            CacheEntry {
                hash: "temp_hash".to_string(),
                object: PathBuf::from("temp.o"),
            },
        );

        assert_eq!(cache.files.len(), 1);

        cache.files.remove("temp.cpp");

        assert!(cache.files.is_empty());
    }

    #[test]
    fn test_cache_with_windows_paths() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let mut files = HashMap::new();
        files.insert(
            "src\\main.cpp".to_string(),
            CacheEntry {
                hash: "win_hash".to_string(),
                object: PathBuf::from("obj\\main.obj"),
            },
        );
        let cache = IncrementalCache { files };

        cache.save(&cache_path)?;
        let loaded_cache = IncrementalCache::load(&cache_path)?;

        assert_eq!(loaded_cache.files.len(), 1);
        assert_eq!(loaded_cache.files["src\\main.cpp"].hash, "win_hash");
        assert_eq!(
            loaded_cache.files["src\\main.cpp"].object,
            PathBuf::from("obj\\main.obj")
        );

        Ok(())
    }
}
