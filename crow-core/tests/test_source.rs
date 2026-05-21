#[cfg(test)]
mod tests {
    use crow_core::builder::paths::SourceFilePath;
    use crow_core::builder::tasks::SourceCompilationTask;
    use crow_core::config::{BuildConfig, CrowConfig, Profile};
    use crow_core::project::Project;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn create_test_project() -> (tempfile::TempDir, Project) {
        let root = tempdir().unwrap();
        let config = CrowConfig {
            package: Some(crow_core::config::Package {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            }),
            workspace: None,
            build: BuildConfig::default(),
            dependencies: Default::default(),
            profile: Default::default(),
        };
        let project = Project::new(config, root.path().to_path_buf(), "dev", None)
            .expect("Failed to create project");
        (root, project)
    }

    fn create_test_build_config() -> BuildConfig {
        BuildConfig::default()
    }

    #[test]
    fn test_prepare_basic() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("test.c"));
        let deps_dir = Path::new("/tmp/deps");
        let build_config = create_test_build_config();
        let profile = Profile::Dev(Default::default());

        let result = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        );

        assert!(result.is_ok());
        let task = result.unwrap();

        assert_eq!(task.source.as_path(), Path::new("test.c"));
        let obj = task.object.as_path().to_string_lossy();
        assert!(
            obj.ends_with(".o") || obj.ends_with(".obj"),
            "unexpected object extension: {obj}"
        );
        assert!(task.dep_file.as_path().to_string_lossy().ends_with(".d"));
    }

    #[test]
    fn test_prepare_with_include_dirs() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("test.c"));
        let deps_dir = Path::new("/tmp/deps");
        let mut build_config = create_test_build_config();
        build_config.include_dirs = vec![PathBuf::from("include"), PathBuf::from("src/include")];
        let profile = Profile::Dev(Default::default());

        let result = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_prepare_with_defines() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("test.c"));
        let deps_dir = Path::new("/tmp/deps");
        let mut build_config = create_test_build_config();
        build_config.preprocessor_defines = vec!["DEBUG=1".to_string(), "FEATURE".to_string()];
        let profile = Profile::Dev(Default::default());

        let result = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_prepare_with_warnings_as_errors() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("test.c"));
        let deps_dir = Path::new("/tmp/deps");
        let mut build_config = create_test_build_config();
        build_config.warnings_as_errors = true;
        let profile = Profile::Dev(Default::default());

        let result = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_prepare_with_release_profile() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("test.c"));
        let deps_dir = Path::new("/tmp/deps");
        let build_config = create_test_build_config();
        let profile = Profile::Release(Default::default());

        let result = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_to_compile_command() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("test.c"));
        let deps_dir = Path::new("/tmp/deps");
        let build_config = create_test_build_config();
        let profile = Profile::Dev(Default::default());

        let task = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        )
        .unwrap();

        let compile_cmd = task.to_compile_command(Path::new("/build/dir"));

        assert_eq!(compile_cmd.directory, "/build/dir");
        assert_eq!(compile_cmd.file, "test.c");
        assert!(compile_cmd.command.contains("gcc"));
        assert!(!compile_cmd.command.is_empty());
    }

    #[test]
    fn test_to_cache_entry() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("test.c"));
        let deps_dir = Path::new("/tmp/deps");
        let build_config = create_test_build_config();
        let profile = Profile::Dev(Default::default());

        let task = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        )
        .unwrap();

        let hash = "abc123def456".to_string();
        let cache_entry = task.to_cache_entry(hash.clone());

        assert_eq!(cache_entry.hash, hash);
        assert_eq!(cache_entry.object, task.object.0);
    }

    #[test]
    fn test_prepare_invalid_source() {
        let (_root, project) = create_test_project();
        let source = SourceFilePath(PathBuf::from("."));
        let deps_dir = Path::new("/tmp/deps");
        let build_config = create_test_build_config();
        let profile = Profile::Dev(Default::default());

        let result = SourceCompilationTask::prepare(
            &source,
            "gcc",
            &build_config,
            deps_dir,
            &project,
            &profile,
            &[],
        );

        assert!(result.is_err());
    }
}
