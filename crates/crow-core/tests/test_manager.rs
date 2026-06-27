#[cfg(test)]
mod tests {
    use crow_core::builder::incremental::CacheEntry;
    use crow_core::builder::incremental::IncrementalManager;
    use crow_core::builder::paths::ObjectFilePath;
    use crow_core::builder::paths::SourceFilePath;
    use std::fs::File;
    use std::path::Path;
    use tempfile::tempdir;

    fn create_src_path(path: &Path, name: &str) -> SourceFilePath {
        SourceFilePath(path.join(name))
    }

    fn create_obj_path(path: &Path, name: &str) -> ObjectFilePath {
        ObjectFilePath(path.join(name))
    }

    fn create_cache(hash: &str, object_path: &Path) -> CacheEntry {
        CacheEntry {
            hash: hash.to_string(),
            object: object_path.to_path_buf(),
        }
    }

    #[test]
    fn test_new_inc_enabled() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let manager = IncrementalManager::new(&cache_path, true);

        let src = create_src_path(dir.path(), "test.c");
        let obj = create_obj_path(dir.path(), "test.o");

        assert!(manager.should_compile(&src, &Some("hash".to_string()), &obj));
    }

    #[test]
    fn test_new_inc_disabled() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let manager = IncrementalManager::new(&cache_path, false);

        let src = create_src_path(dir.path(), "test.c");
        let obj = create_obj_path(dir.path(), "test.o");

        assert!(manager.should_compile(&src, &None, &obj));
        assert!(manager.should_compile(&src, &Some("hash".to_string()), &obj));
    }

    #[test]
    fn test_compile_inc_disabled() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, false);

        assert!(manager.should_compile(&src, &None, &obj));
        assert!(manager.should_compile(&src, &Some("hash".to_string()), &obj));
    }

    #[test]
    fn test_compile_first() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, true);

        assert!(manager.should_compile(&src, &Some("hash123".to_string()), &obj));
    }

    #[test]
    fn test_compile_no_hash() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, true);

        let entry = create_cache("hash123", &obj.as_path());
        manager.record_success(&src, entry);

        assert!(manager.should_compile(&src, &None, &obj));
    }

    #[test]
    fn test_compile_mismatch() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, true);

        let entry = create_cache("old_hash", &obj.as_path());
        manager.record_success(&src, entry);

        assert!(manager.should_compile(&src, &Some("new_hash".to_string()), &obj));
    }

    #[test]
    fn test_compile_match_exists() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        File::create(&obj.as_path()).unwrap();

        let manager = IncrementalManager::new(&cache_path, true);

        let entry = create_cache("same_hash", &obj.as_path());
        manager.record_success(&src, entry);

        assert!(!manager.should_compile(&src, &Some("same_hash".to_string()), &obj));
    }

    #[test]
    fn test_compile_match_missing() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, true);

        let entry = create_cache("same_hash", &obj.as_path());
        manager.record_success(&src, entry);

        assert!(manager.should_compile(&src, &Some("same_hash".to_string()), &obj));
    }

    #[test]
    fn test_compile_diff_path() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj_saved = create_obj_path(dir.path(), "old.o");
        let obj_new = create_obj_path(dir.path(), "new.o");

        File::create(&obj_new.as_path()).unwrap();

        let manager = IncrementalManager::new(&cache_path, true);

        let entry = create_cache("same_hash", &obj_saved.as_path());
        manager.record_success(&src, entry);

        assert!(manager.should_compile(&src, &Some("same_hash".to_string()), &obj_new));
    }

    #[test]
    fn test_record_inc_disabled() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, false);

        let entry = create_cache("hash123", &obj.as_path());
        manager.record_success(&src, entry);

        assert!(manager.should_compile(&src, &Some("hash123".to_string()), &obj));
        assert!(manager.should_compile(&src, &None, &obj));
    }

    #[test]
    fn test_record_inc_enabled() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, true);

        let entry = create_cache("hash123", &obj.as_path());
        manager.record_success(&src, entry);

        File::create(&obj.as_path()).unwrap();
        assert!(!manager.should_compile(&src, &Some("hash123".to_string()), &obj));
    }

    #[test]
    fn test_record_overwrites() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj1 = create_obj_path(dir.path(), "old.o");
        let obj2 = create_obj_path(dir.path(), "new.o");

        let manager = IncrementalManager::new(&cache_path, true);

        let entry1 = create_cache("hash123", &obj1.as_path());
        manager.record_success(&src, entry1);

        let entry2 = create_cache("hash456", &obj2.as_path());
        manager.record_success(&src, entry2);

        File::create(&obj2.as_path()).unwrap();
        assert!(!manager.should_compile(&src, &Some("hash456".to_string()), &obj2));

        assert!(manager.should_compile(&src, &Some("hash123".to_string()), &obj1));
    }

    #[test]
    fn test_record_multiple() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src1 = create_src_path(dir.path(), "main.c");
        let src2 = create_src_path(dir.path(), "utils.c");
        let obj1 = create_obj_path(dir.path(), "main.o");
        let obj2 = create_obj_path(dir.path(), "utils.o");

        let manager = IncrementalManager::new(&cache_path, true);

        manager.record_success(&src1, create_cache("hash1", &obj1.as_path()));
        manager.record_success(&src2, create_cache("hash2", &obj2.as_path()));

        File::create(&obj1.as_path()).unwrap();
        File::create(&obj2.as_path()).unwrap();

        assert!(!manager.should_compile(&src1, &Some("hash1".to_string()), &obj1));
        assert!(!manager.should_compile(&src2, &Some("hash2".to_string()), &obj2));
    }

    #[test]
    fn test_finalize_inc_disabled() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let manager1 = IncrementalManager::new(&cache_path, true);
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");
        manager1.record_success(&src, create_cache("hash", &obj.as_path()));
        manager1.finalize()?;

        assert!(cache_path.exists());

        let manager2 = IncrementalManager::new(&cache_path, false);
        manager2.finalize()?;

        assert!(!cache_path.exists());

        Ok(())
    }

    #[test]
    fn test_finalize_inc_enabled() -> anyhowed::Result<()> {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");

        let manager = IncrementalManager::new(&cache_path, true);
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");
        manager.record_success(&src, create_cache("test_hash", &obj.as_path()));
        manager.finalize()?;

        assert!(cache_path.exists());

        let new_manager = IncrementalManager::new(&cache_path, true);
        File::create(&obj.as_path()).unwrap();

        assert!(!new_manager.should_compile(&src, &Some("test_hash".to_string()), &obj));

        Ok(())
    }

    #[test]
    fn test_concurrent() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let manager = std::sync::Arc::new(IncrementalManager::new(&cache_path, true));

        let mut handles = vec![];

        for i in 0..10 {
            let manager_clone = manager.clone();
            let dir_clone = dir.path().to_path_buf();

            let handle = std::thread::spawn(move || {
                let src = create_src_path(&dir_clone, &format!("file{}.c", i));
                let obj = create_obj_path(&dir_clone, &format!("file{}.o", i));
                let hash = format!("hash{}", i);

                assert!(manager_clone.should_compile(&src, &Some(hash.clone()), &obj));

                manager_clone.record_success(&src, create_cache(&hash, &obj.as_path()));

                File::create(&obj.as_path()).unwrap();
                assert!(!manager_clone.should_compile(&src, &Some(hash), &obj));
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_compile_real_ops() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let src = create_src_path(dir.path(), "main.c");
        let obj = create_obj_path(dir.path(), "main.o");

        let manager = IncrementalManager::new(&cache_path, true);

        assert!(manager.should_compile(&src, &Some("hash1".to_string()), &obj));
        manager.record_success(&src, create_cache("hash1", &obj.as_path()));

        File::create(&obj.as_path()).unwrap();

        assert!(!manager.should_compile(&src, &Some("hash1".to_string()), &obj));

        std::fs::remove_file(&obj.as_path()).unwrap();

        assert!(manager.should_compile(&src, &Some("hash1".to_string()), &obj));

        assert!(manager.should_compile(&src, &Some("hash2".to_string()), &obj));
    }
}
