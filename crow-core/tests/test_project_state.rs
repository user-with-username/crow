#[cfg(test)]
mod tests {
    use crow_core::builder::incremental::BuildState;
    use std::fs;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_build_state_default() {
        let state = BuildState::default();

        assert_eq!(state.build_hash, "");
        assert_eq!(state.lock_hash, "");
        assert_eq!(state.output, None);
    }

    #[test]
    fn test_build_state_new() {
        let state = BuildState {
            build_hash: "abc123".to_string(),
            lock_hash: "def456".to_string(),
            output: Some(PathBuf::from("build/output.exe")),
        };

        assert_eq!(state.build_hash, "abc123");
        assert_eq!(state.lock_hash, "def456");
        assert_eq!(state.output, Some(PathBuf::from("build/output.exe")));
    }

    #[test]
    fn test_load_or_default_with_nonexistent_file() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("nonexistent_state.json");

        let state = BuildState::load_or_default(&state_path);

        assert_eq!(state.build_hash, "");
        assert_eq!(state.lock_hash, "");
        assert_eq!(state.output, None);
    }

    #[test]
    fn test_load_or_default_with_existing_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");

        let state = BuildState {
            build_hash: "build123".to_string(),
            lock_hash: "lock456".to_string(),
            output: Some(PathBuf::from("output/app.exe")),
        };
        state.save(&state_path)?;

        let loaded_state = BuildState::load_or_default(&state_path);

        assert_eq!(loaded_state.build_hash, "build123");
        assert_eq!(loaded_state.lock_hash, "lock456");
        assert_eq!(loaded_state.output, Some(PathBuf::from("output/app.exe")));

        Ok(())
    }

    #[test]
    fn test_load_with_nonexistent_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("missing.json");

        let state = BuildState::load(&state_path)?;

        assert_eq!(state.build_hash, "");
        assert_eq!(state.lock_hash, "");
        assert_eq!(state.output, None);

        Ok(())
    }

    #[test]
    fn test_load_with_existing_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");

        let state = BuildState {
            build_hash: "test_build".to_string(),
            lock_hash: "test_lock".to_string(),
            output: Some(PathBuf::from("test/output.o")),
        };
        state.save(&state_path)?;

        let loaded_state = BuildState::load(&state_path)?;

        assert_eq!(loaded_state.build_hash, "test_build");
        assert_eq!(loaded_state.lock_hash, "test_lock");
        assert_eq!(loaded_state.output, Some(PathBuf::from("test/output.o")));

        Ok(())
    }

    #[test]
    fn test_load_with_invalid_json() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("invalid.json");

        fs::write(&state_path, "this is not valid json")?;

        let result = BuildState::load(&state_path);

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_save_state() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("saved_state.json");

        let state = BuildState {
            build_hash: "save_test".to_string(),
            lock_hash: "lock_test".to_string(),
            output: Some(PathBuf::from("output/saved.exe")),
        };

        state.save(&state_path)?;

        assert!(state_path.exists());

        let contents = fs::read_to_string(&state_path)?;
        assert!(contents.contains("save_test"));
        assert!(contents.contains("lock_test"));
        assert!(contents.contains("output/saved.exe"));

        Ok(())
    }

    #[test]
    fn test_save_and_load_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("roundtrip.json");

        let original_state = BuildState {
            build_hash: "round_build".to_string(),
            lock_hash: "round_lock".to_string(),
            output: Some(PathBuf::from("round/output.exe")),
        };

        original_state.save(&state_path)?;
        let loaded_state = BuildState::load(&state_path)?;

        assert_eq!(original_state.build_hash, loaded_state.build_hash);
        assert_eq!(original_state.lock_hash, loaded_state.lock_hash);
        assert_eq!(original_state.output, loaded_state.output);

        Ok(())
    }

    #[test]
    fn test_requires_rebuild_with_incremental_disabled() {
        let state = BuildState {
            build_hash: "hash1".to_string(),
            lock_hash: "lock1".to_string(),
            output: Some(PathBuf::from("output.exe")),
        };

        assert!(state.requires_rebuild("hash1", "lock1", Some(Path::new("output.exe")), false));
        assert!(state.requires_rebuild(
            "different",
            "different",
            Some(Path::new("output.exe")),
            false
        ));
    }

    #[test]
    fn test_requires_rebuild_with_matching_hashes_and_output(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.exe");

        File::create(&output_path)?;

        let state = BuildState {
            build_hash: "hash123".to_string(),
            lock_hash: "lock456".to_string(),
            output: Some(output_path.clone()),
        };

        assert!(!state.requires_rebuild("hash123", "lock456", Some(&output_path), true));

        Ok(())
    }

    #[test]
    fn test_requires_rebuild_with_different_build_hash() {
        let state = BuildState {
            build_hash: "old_build".to_string(),
            lock_hash: "lock456".to_string(),
            output: Some(PathBuf::from("output.exe")),
        };

        assert!(state.requires_rebuild(
            "new_build",
            "lock456",
            Some(Path::new("output.exe")),
            true
        ));
    }

    #[test]
    fn test_requires_rebuild_with_different_lock_hash() {
        let state = BuildState {
            build_hash: "hash123".to_string(),
            lock_hash: "old_lock".to_string(),
            output: Some(PathBuf::from("output.exe")),
        };

        assert!(state.requires_rebuild("hash123", "new_lock", Some(Path::new("output.exe")), true));
    }

    #[test]
    fn test_requires_rebuild_with_missing_output_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("missing.exe");

        let state = BuildState {
            build_hash: "hash123".to_string(),
            lock_hash: "lock456".to_string(),
            output: Some(output_path.clone()),
        };

        assert!(state.requires_rebuild("hash123", "lock456", Some(&output_path), true));

        Ok(())
    }

    #[test]
    fn test_requires_rebuild_with_different_output_path() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempdir().unwrap();
        let output_path1 = dir.path().join("output1.exe");
        let output_path2 = dir.path().join("output2.exe");

        File::create(&output_path1)?;

        let state = BuildState {
            build_hash: "hash123".to_string(),
            lock_hash: "lock456".to_string(),
            output: Some(output_path1),
        };

        assert!(state.requires_rebuild("hash123", "lock456", Some(&output_path2), true));

        Ok(())
    }

    #[test]
    fn test_requires_rebuild_with_no_saved_output_and_missing_output() {
        let state = BuildState {
            build_hash: "hash123".to_string(),
            lock_hash: "lock456".to_string(),
            output: None,
        };

        let output_path = Path::new("output.exe");
        assert!(!output_path.exists());
        assert!(state.requires_rebuild("hash123", "lock456", Some(output_path), true));
    }

    #[test]
    fn test_requires_rebuild_with_no_output_path_provided() {
        let state = BuildState {
            build_hash: "hash123".to_string(),
            lock_hash: "lock456".to_string(),
            output: Some(PathBuf::from("output.exe")),
        };

        assert!(!state.requires_rebuild("hash123", "lock456", None, true));
    }

    #[test]
    fn test_requires_rebuild_with_all_matching_and_existing_file(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("existing.exe");

        File::create(&output_path)?;

        let state = BuildState {
            build_hash: "match".to_string(),
            lock_hash: "match".to_string(),
            output: Some(output_path.clone()),
        };

        assert!(!state.requires_rebuild("match", "match", Some(&output_path), true));

        Ok(())
    }

    #[test]
    fn test_requires_rebuild_with_empty_hashes() {
        let state = BuildState {
            build_hash: "".to_string(),
            lock_hash: "".to_string(),
            output: None,
        };

        assert!(!state.requires_rebuild("", "", None, true));
        assert!(state.requires_rebuild("", "lock", None, true));
        assert!(state.requires_rebuild("build", "", None, true));
    }

    #[test]
    fn test_clone() {
        let state1 = BuildState {
            build_hash: "hash_clone".to_string(),
            lock_hash: "lock_clone".to_string(),
            output: Some(PathBuf::from("clone.exe")),
        };

        let state2 = state1.clone();

        assert_eq!(state1.build_hash, state2.build_hash);
        assert_eq!(state1.lock_hash, state2.lock_hash);
        assert_eq!(state1.output, state2.output);
    }

    #[test]
    fn test_debug_formatting() {
        let state = BuildState {
            build_hash: "debug_hash".to_string(),
            lock_hash: "debug_lock".to_string(),
            output: Some(PathBuf::from("debug.exe")),
        };

        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("debug_hash"));
        assert!(debug_str.contains("debug_lock"));
        assert!(debug_str.contains("debug.exe"));
    }

    #[test]
    fn test_serialization_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let original = BuildState {
            build_hash: "serial_test".to_string(),
            lock_hash: "serial_lock".to_string(),
            output: Some(PathBuf::from("serial/output.exe")),
        };

        let serialized = serde_json::to_string_pretty(&original)?;
        let deserialized: BuildState = serde_json::from_str(&serialized)?;

        assert_eq!(original.build_hash, deserialized.build_hash);
        assert_eq!(original.lock_hash, deserialized.lock_hash);
        assert_eq!(original.output, deserialized.output);

        Ok(())
    }

    #[test]
    fn test_empty_state_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let empty_state = BuildState::default();

        let serialized = serde_json::to_string(&empty_state)?;
        let deserialized: BuildState = serde_json::from_str(&serialized)?;

        assert_eq!(deserialized.build_hash, "");
        assert_eq!(deserialized.lock_hash, "");
        assert_eq!(deserialized.output, None);

        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_requires_rebuild_with_symlinks() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let real_path = dir.path().join("real.exe");
        let symlink_path = dir.path().join("link.exe");

        File::create(&real_path)?;

        std::os::unix::fs::symlink(&real_path, &symlink_path)?;

        let state = BuildState {
            build_hash: "hash".to_string(),
            lock_hash: "lock".to_string(),
            output: Some(symlink_path.clone()),
        };

        assert!(!state.requires_rebuild("hash", "lock", Some(&symlink_path), true));

        Ok(())
    }

    #[test]
    fn test_update_state_fields() {
        let mut state = BuildState::default();

        state.build_hash = "new_build".to_string();
        state.lock_hash = "new_lock".to_string();
        state.output = Some(PathBuf::from("new_output.exe"));

        assert_eq!(state.build_hash, "new_build");
        assert_eq!(state.lock_hash, "new_lock");
        assert_eq!(state.output, Some(PathBuf::from("new_output.exe")));
    }
}
