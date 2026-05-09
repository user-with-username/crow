#[cfg(test)]
mod tests {
    use crow_core::builder::tasks::database::CompilationDatabase;
    use crow_core::config::{BuildConfig, CrowConfig, Package};
    use crow_core::project::Project;
    use std::path::Path;
    use tempfile::tempdir;

    fn create_test_project(root: &Path) -> Project {
        let config = CrowConfig {
            package: Some(Package {
                name: "test_project".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            }),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Default::default(),
            profile: Default::default(),
        };
        Project::new(config, root.to_path_buf(), "dev", None).expect("Failed to create test project")
    }

    fn args_as_stored(project: &Project, base: Vec<String>) -> Vec<String> {
        let mut args = base;
        if !args.is_empty() && project.compiler_kind().is_msvc() {
            args.insert(1, "--driver-mode=cl".to_string());
        }
        args
    }

    #[test]
    fn test_clean_path_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.c");
        std::fs::write(&file_path, "// test").unwrap();

        let cleaned = CompilationDatabase::clean_path(&file_path);

        assert!(cleaned.is_ascii());
        assert!(!cleaned.starts_with(r"\\?\"));
        assert!(cleaned.ends_with("test.c"));
        assert!(std::path::Path::new(&cleaned).is_absolute());
    }

    #[test]
    fn test_clean_path_non_existent() {
        let path = Path::new("/nonexistent/path/test.c");
        let cleaned = CompilationDatabase::clean_path(path);

        assert!(!cleaned.is_empty());
        assert!(cleaned.contains("test.c"));
    }

    #[test]
    fn test_clean_path_windows_prefix() {
        let path = Path::new(r"\\?\C:\test\file.c");
        let cleaned = CompilationDatabase::clean_path(path);

        assert!(!cleaned.starts_with(r"\\?\"));
    }

    #[test]
    fn test_add_entry_empty_args() {
        let dir = tempdir().unwrap();
        let project = create_test_project(dir.path());
        let mut db = CompilationDatabase::new();

        let file_path = Path::new("empty.c");
        let args = vec![];

        db.add_entry(&project, file_path, args);

        let temp = tempdir().unwrap();
        db.save(temp.path()).expect("Save failed");

        let content = std::fs::read_to_string(temp.path().join("compile_commands.json")).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.len(), 1);
        let json_args = parsed[0]["arguments"].as_array().unwrap();
        assert!(json_args.is_empty());
    }

    #[test]
    fn test_add_entry_relative_path() {
        let dir = tempdir().unwrap();
        let project = create_test_project(dir.path());
        let mut db = CompilationDatabase::new();

        let file_path = Path::new("../../lib/dep.c");
        let args = vec![
            "gcc".to_string(),
            "-c".to_string(),
            "../../lib/dep.c".to_string(),
        ];

        db.add_entry(&project, file_path, args);

        let temp = tempdir().unwrap();
        db.save(temp.path()).expect("Save failed");

        let content = std::fs::read_to_string(temp.path().join("compile_commands.json")).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

        let file_path_str = parsed[0]["file"].as_str().unwrap();
        assert!(std::path::Path::new(file_path_str).is_absolute());
        assert!(file_path_str.ends_with("dep.c"));
    }

    #[test]
    fn test_add_multiple_entries() {
        let dir = tempdir().unwrap();
        let project = create_test_project(dir.path());
        let mut db = CompilationDatabase::new();

        let files = vec![
            (
                "src/main.c",
                vec![
                    "gcc".to_string(),
                    "-c".to_string(),
                    "src/main.c".to_string(),
                ],
            ),
            (
                "src/util.c",
                vec![
                    "gcc".to_string(),
                    "-c".to_string(),
                    "src/util.c".to_string(),
                ],
            ),
            (
                "src/parser.c",
                vec![
                    "gcc".to_string(),
                    "-c".to_string(),
                    "src/parser.c".to_string(),
                ],
            ),
        ];

        for (file, args) in &files {
            db.add_entry(&project, Path::new(file), args.clone());
        }

        let temp = tempdir().unwrap();
        db.save(temp.path()).expect("Save failed");

        let content = std::fs::read_to_string(temp.path().join("compile_commands.json")).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.len(), 3);

        for (i, (file, args)) in files.iter().enumerate() {
            let json_args: Vec<String> = parsed[i]["arguments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();

            assert_eq!(json_args, args_as_stored(&project, args.clone()));
            let expected_file = CompilationDatabase::clean_path(project.root.join(file));
            assert_eq!(parsed[i]["file"].as_str().unwrap(), expected_file);
        }
    }

    #[test]
    fn test_save_and_read() {
        let dir = tempdir().unwrap();
        let project = create_test_project(dir.path());
        let mut db = CompilationDatabase::new();

        let file_path = Path::new("src/main.c");
        let args = vec![
            "gcc".to_string(),
            "-c".to_string(),
            "-Wall".to_string(),
            "-O2".to_string(),
            "src/main.c".to_string(),
            "-o".to_string(),
            "build/main.o".to_string(),
        ];

        db.add_entry(&project, file_path, args.clone());

        let json_path = dir.path().join("compile_commands.json");
        db.save(dir.path()).expect("Failed to save database");

        assert!(json_path.exists());

        let content = std::fs::read_to_string(&json_path).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0]["directory"].as_str().unwrap(),
            CompilationDatabase::clean_path(project.root.clone())
        );
        assert_eq!(
            parsed[0]["file"].as_str().unwrap(),
            CompilationDatabase::clean_path(project.root.join(file_path))
        );

        let json_args: Vec<String> = parsed[0]["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();

        assert_eq!(json_args, args_as_stored(&project, args));
    }

    #[test]
    fn test_save_empty_database() {
        let dir = tempdir().unwrap();
        let db = CompilationDatabase::new();

        db.save(dir.path()).expect("Failed to save empty database");

        let json_path = dir.path().join("compile_commands.json");
        assert!(json_path.exists());

        let content = std::fs::read_to_string(&json_path).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_save_overwrites_existing() {
        let dir = tempdir().unwrap();
        let project = create_test_project(dir.path());
        let json_path = dir.path().join("compile_commands.json");

        let mut db1 = CompilationDatabase::new();
        db1.add_entry(
            &project,
            Path::new("first.c"),
            vec!["gcc".to_string(), "first.c".to_string()],
        );
        db1.save(dir.path()).unwrap();

        let first_content = std::fs::read_to_string(&json_path).unwrap();
        assert!(first_content.contains("first.c"));

        let mut db2 = CompilationDatabase::new();
        db2.add_entry(
            &project,
            Path::new("second.c"),
            vec!["gcc".to_string(), "second.c".to_string()],
        );
        db2.save(dir.path()).unwrap();

        let second_content = std::fs::read_to_string(&json_path).unwrap();
        assert!(!second_content.contains("first.c"));
        assert!(second_content.contains("second.c"));
    }

    #[test]
    fn test_clean_path_special_characters() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("my project");
        std::fs::create_dir(&subdir).unwrap();

        let file = subdir.join("test file.c");
        std::fs::write(&file, "// test").unwrap();

        let cleaned = CompilationDatabase::clean_path(&file);
        assert!(cleaned.contains("my project"));
        assert!(cleaned.contains("test file.c"));
    }

    #[test]
    fn test_clean_path_symlink() {
        let dir = tempdir().unwrap();
        let original = dir.path().join("original.c");
        let symlink = dir.path().join("link.c");

        std::fs::write(&original, "// test").unwrap();

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&original, &symlink).unwrap();
            let cleaned = CompilationDatabase::clean_path(&symlink);
            assert!(cleaned.ends_with("original.c"));
        }

        if symlink.exists() {
            let cleaned = CompilationDatabase::clean_path(&symlink);
            assert!(!cleaned.is_empty());
        }
    }
}
