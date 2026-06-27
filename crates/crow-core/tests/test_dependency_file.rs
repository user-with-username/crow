#[cfg(test)]
mod tests {
    use crow_core::builder::paths::DependencyFileNaming;
    use crow_core::builder::paths::DependencyFilePath;
    use crow_core::builder::paths::SourceFilePath;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_dependency_file_path_from_pathbuf() {
        let path_buf = PathBuf::from("/some/path/file.d");
        let dep_path = DependencyFilePath::from(path_buf.clone());
        assert_eq!(dep_path.as_path(), path_buf.as_path());
    }

    #[test]
    fn test_dependency_file_path_from_path_ref() {
        let path = Path::new("/some/path/file.d");
        let dep_path = DependencyFilePath::from(path);
        assert_eq!(dep_path.as_path(), path);
    }

    #[test]
    fn test_dependency_file_path_from_string() {
        let path_str = "/some/path/file.d".to_string();
        let dep_path = DependencyFilePath::from(path_str.clone());
        assert_eq!(dep_path.as_path(), Path::new(&path_str));
    }

    #[test]
    fn test_dependency_file_path_from_str() {
        let path_str = "/some/path/file.d";
        let dep_path = DependencyFilePath::from(path_str);
        assert_eq!(dep_path.as_path(), Path::new(path_str));
    }

    #[test]
    fn test_dependency_file_path_into_pathbuf() {
        let path_buf = PathBuf::from("/some/path/file.d");
        let dep_path = DependencyFilePath(path_buf.clone());
        let converted: PathBuf = dep_path.into();
        assert_eq!(converted, path_buf);
    }

    #[test]
    fn test_as_path() {
        let path = PathBuf::from("/test/path/file.d");
        let dep_path = DependencyFilePath(path.clone());
        assert_eq!(dep_path.as_path(), path.as_path());
    }

    #[test]
    fn test_exists_returns_true_when_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("exists.d");
        File::create(&file_path).unwrap();

        let dep_path = DependencyFilePath(file_path);
        assert!(dep_path.exists());
    }

    #[test]
    fn test_exists_returns_false_when_file_does_not_exist() {
        let dep_path = DependencyFilePath(PathBuf::from("/nonexistent/path/file.d"));
        assert!(!dep_path.exists());
    }

    #[test]
    fn test_parent_with_parent() {
        let path = PathBuf::from("/parent/child/file.d");
        let dep_path = DependencyFilePath(path);
        assert_eq!(dep_path.parent(), Some(Path::new("/parent/child")));
    }

    #[test]
    fn test_parent_without_parent() {
        let path = PathBuf::from("file.d");
        let dep_path = DependencyFilePath(path);
        let parent = dep_path.parent();
        match parent {
            Some(p) if p.as_os_str().is_empty() => {}
            None => {}
            _ => panic!("Expected None or Some(empty path), got {:?}", parent),
        }
    }

    #[test]
    fn test_file_name_with_valid_name() {
        let path = PathBuf::from("/some/path/file.d");
        let dep_path = DependencyFilePath(path);
        assert_eq!(dep_path.file_name(), Some("file.d"));
    }

    #[test]
    fn test_file_name_with_invalid_unicode() {
        #[cfg(unix)]
        {
            use std::ffi::OsString;
            use std::os::unix::ffi::OsStringExt;
            let invalid_bytes = vec![0xff, 0xfe, 0xfd];
            let os_string = OsString::from_vec(invalid_bytes);
            let path = PathBuf::from(os_string);
            let dep_path = DependencyFilePath(path);
            assert_eq!(dep_path.file_name(), None);
        }
    }

    #[test]
    fn test_file_stem_with_extension() {
        let path = PathBuf::from("/some/path/file.d");
        let dep_path = DependencyFilePath(path);
        assert_eq!(dep_path.file_stem(), Some("file"));
    }

    #[test]
    fn test_file_stem_without_extension() {
        let path = PathBuf::from("/some/path/file");
        let dep_path = DependencyFilePath(path);
        assert_eq!(dep_path.file_stem(), Some("file"));
    }

    #[test]
    fn test_extension_with_extension() {
        let path = PathBuf::from("/some/path/file.d");
        let dep_path = DependencyFilePath(path);
        assert_eq!(dep_path.extension(), Some("d"));
    }

    #[test]
    fn test_extension_without_extension() {
        let path = PathBuf::from("/some/path/file");
        let dep_path = DependencyFilePath(path);
        assert_eq!(dep_path.extension(), None);
    }

    #[test]
    fn test_extension_with_multiple_extensions() {
        let path = PathBuf::from("/some/path/file.tar.gz");
        let dep_path = DependencyFilePath(path);
        assert_eq!(dep_path.extension(), Some("gz"));
    }

    #[test]
    fn test_into_path_buf() {
        let path_buf = PathBuf::from("/some/path/file.d");
        let dep_path = DependencyFilePath(path_buf.clone());
        assert_eq!(dep_path.into_path_buf(), path_buf);
    }

    #[test]
    fn test_clone() {
        let path = PathBuf::from("/some/path/file.d");
        let dep_path1 = DependencyFilePath(path);
        let dep_path2 = dep_path1.clone();
        assert_eq!(dep_path1.as_path(), dep_path2.as_path());
    }

    #[test]
    fn test_debug_format() {
        let path = PathBuf::from("/some/path/file.d");
        let dep_path = DependencyFilePath(path);
        let debug_str = format!("{:?}", dep_path);
        assert!(debug_str.contains("DependencyFilePath"));
        assert!(debug_str.contains("file.d"));
    }

    #[test]
    fn test_generate_dependency_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = SourceFilePath(PathBuf::from("/src/main.rs"));

        let dep_path = DependencyFileNaming::generate(temp_dir.path(), &source);

        let expected = temp_dir.path().join("main.d");
        assert_eq!(dep_path.as_path(), expected);
    }

    #[test]
    fn test_generate_with_custom_stem() {
        let temp_dir = TempDir::new().unwrap();
        let source = SourceFilePath(PathBuf::from("/src/my_module.rs"));

        let dep_path = DependencyFileNaming::generate(temp_dir.path(), &source);

        let expected = temp_dir.path().join("my_module.d");
        assert_eq!(dep_path.as_path(), expected);
    }

    #[test]
    fn test_generate_with_source_having_extension() {
        let temp_dir = TempDir::new().unwrap();
        let source = SourceFilePath(PathBuf::from("/src/component.cpp"));

        let dep_path = DependencyFileNaming::generate(temp_dir.path(), &source);

        let expected = temp_dir.path().join("component.d");
        assert_eq!(dep_path.as_path(), expected);
    }

    #[test]
    fn test_generate_with_absolute_path() {
        let deps_dir = Path::new("/absolute/deps/dir");
        let source = SourceFilePath(PathBuf::from("/absolute/src/file.rs"));

        let dep_path = DependencyFileNaming::generate(deps_dir, &source);

        let expected = deps_dir.join("file.d");
        assert_eq!(dep_path.as_path(), expected);
    }

    #[test]
    fn test_generate_with_relative_path() {
        let deps_dir = Path::new("deps");
        let source = SourceFilePath(PathBuf::from("src/file.rs"));

        let dep_path = DependencyFileNaming::generate(deps_dir, &source);

        let expected = deps_dir.join("file.d");
        assert_eq!(dep_path.as_path(), expected);
    }
}
