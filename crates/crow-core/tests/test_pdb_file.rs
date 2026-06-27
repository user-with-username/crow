#[cfg(test)]
mod tests {
    use crow_core::builder::paths::PdbFileNaming;
    use crow_core::builder::paths::PdbFilePath;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_pdb_file_path_as_path() {
        let path = PathBuf::from("/tmp/test.pdb");
        let pdb_path = PdbFilePath::from(path.clone());
        assert_eq!(pdb_path.as_path(), path.as_path());
    }

    #[test]
    fn test_pdb_file_path_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("exists.pdb");
        File::create(&file_path).unwrap();

        let pdb_path = PdbFilePath::from(file_path);
        assert!(pdb_path.exists());

        let non_existent = PdbFilePath::from("/tmp/non_existent_xyz.pdb");
        assert!(!non_existent.exists());
    }

    #[test]
    fn test_pdb_file_path_parent() {
        let path = PathBuf::from("/foo/bar/test.pdb");
        let pdb_path = PdbFilePath::from(path);
        assert_eq!(pdb_path.parent(), Some(Path::new("/foo/bar")));

        let root_path = PdbFilePath::from("/test.pdb");
        assert_eq!(root_path.parent(), Some(Path::new("/")));
    }

    #[test]
    fn test_pdb_file_path_file_name() {
        let path = PathBuf::from("/foo/bar/test.pdb");
        let pdb_path = PdbFilePath::from(path);
        assert_eq!(pdb_path.file_name(), Some("test.pdb"));

        let invalid_path = PdbFilePath::from(std::path::Path::new("/invalid/\0"));
        let _ = invalid_path.file_name();
    }

    #[test]
    fn test_pdb_file_path_extension() {
        let path = PathBuf::from("/foo/bar/test.pdb");
        let pdb_path = PdbFilePath::from(path);
        assert_eq!(pdb_path.extension(), Some("pdb"));

        let no_extension = PdbFilePath::from("/foo/bar/test");
        assert_eq!(no_extension.extension(), None);

        let multiple_dots = PdbFilePath::from("/foo/bar/test.pdb.bak");
        assert_eq!(multiple_dots.extension(), Some("bak"));
    }

    #[test]
    fn test_pdb_file_path_into_path_buf() {
        let path = PathBuf::from("/tmp/test.pdb");
        let pdb_path = PdbFilePath::from(path.clone());
        assert_eq!(pdb_path.into_path_buf(), path);
    }

    #[test]
    fn test_pdb_file_path_to_string_lossy() {
        let path = PathBuf::from("/tmp/тест.pdb");
        let pdb_path = PdbFilePath::from(path);
        let lossy = pdb_path.to_string_lossy();
        assert!(lossy.contains("тест.pdb"));
    }

    #[test]
    fn test_from_path_buf() {
        let path = PathBuf::from("/test.pdb");
        let pdb_path = PdbFilePath::from(path.clone());
        assert_eq!(pdb_path.0, path);
    }

    #[test]
    fn test_from_path_ref() {
        let path = Path::new("/test.pdb");
        let pdb_path = PdbFilePath::from(path);
        assert_eq!(pdb_path.0, PathBuf::from("/test.pdb"));
    }

    #[test]
    fn test_from_string() {
        let path_str = "/test.pdb".to_string();
        let pdb_path = PdbFilePath::from(path_str);
        assert_eq!(pdb_path.0, PathBuf::from("/test.pdb"));
    }

    #[test]
    fn test_from_str() {
        let pdb_path = PdbFilePath::from("/test.pdb");
        assert_eq!(pdb_path.0, PathBuf::from("/test.pdb"));
    }

    #[test]
    fn test_into_path_buf_for_pdb_file_path() {
        let pdb_path = PdbFilePath::from("/test.pdb");
        let path_buf: PathBuf = pdb_path.into();
        assert_eq!(path_buf, PathBuf::from("/test.pdb"));
    }

    #[test]
    fn test_pdb_file_naming_generate() {
        let pdb_path = PdbFileNaming::generate("/tmp", "myproject");
        assert_eq!(pdb_path.0, PathBuf::from("/tmp/myproject.pdb"));
        assert_eq!(pdb_path.file_name(), Some("myproject.pdb"));
        assert_eq!(pdb_path.file_stem(), Some("myproject"));
        assert_eq!(pdb_path.extension(), Some("pdb"));
    }

    #[test]
    fn test_pdb_file_naming_generate_with_path_buf() {
        let dir = PathBuf::from("/usr/local/bin");
        let pdb_path = PdbFileNaming::generate(dir, "app");
        assert_eq!(pdb_path.0, PathBuf::from("/usr/local/bin/app.pdb"));
    }

    #[test]
    fn test_pdb_file_naming_generate_with_trailing_slash() {
        let pdb_path = PdbFileNaming::generate("/tmp/", "project");
        assert_eq!(pdb_path.0, PathBuf::from("/tmp/project.pdb"));
    }

    #[test]
    fn test_pdb_file_naming_generate_with_complex_project_name() {
        let pdb_path = PdbFileNaming::generate("/tmp", "my-awesome-project_v2");
        assert_eq!(pdb_path.0, PathBuf::from("/tmp/my-awesome-project_v2.pdb"));
        assert_eq!(pdb_path.file_name(), Some("my-awesome-project_v2.pdb"));
    }

    #[test]
    fn test_clone() {
        let pdb_path = PdbFilePath::from("/test.pdb");
        let cloned = pdb_path.clone();
        assert_eq!(pdb_path.0, cloned.0);
    }

    #[test]
    fn test_debug_format() {
        let pdb_path = PdbFilePath::from("/test.pdb");
        let debug_str = format!("{:?}", pdb_path);
        assert!(debug_str.contains("/test.pdb"));
    }
}
