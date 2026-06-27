#[cfg(test)]
mod tests {
    use crow_core::builder::kinds::compiler_kind::CompilerKind;
    use crow_core::builder::paths::ObjectFileNaming;
    use crow_core::builder::paths::ObjectFilePath;
    use crow_core::builder::paths::SourceFilePath;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_object_file_path_as_path() {
        let path = ObjectFilePath(PathBuf::from("/tmp/test.o"));
        assert_eq!(path.as_path(), Path::new("/tmp/test.o"));
    }

    #[test]
    fn test_object_file_path_exists_true() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("exists.o");
        File::create(&file_path).unwrap();

        let obj_path = ObjectFilePath(file_path);
        assert!(obj_path.exists());
    }

    #[test]
    fn test_object_file_path_exists_false() {
        let obj_path = ObjectFilePath(PathBuf::from("/nonexistent/path/file.o"));
        assert!(!obj_path.exists());
    }

    #[test]
    fn test_generate_with_msvc() {
        let deps_dir = Path::new("/build/deps");
        let source = SourceFilePath(PathBuf::from("src/main.rs"));

        let result = ObjectFileNaming::generate(deps_dir, &source, CompilerKind::Msvc);

        assert_eq!(result, PathBuf::from("/build/deps/main.obj"));
    }

    #[test]
    fn test_generate_with_gcc() {
        let deps_dir = Path::new("/build/deps");
        let source = SourceFilePath(PathBuf::from("src/lib.c"));

        let result = ObjectFileNaming::generate(deps_dir, &source, CompilerKind::Gcc);

        assert_eq!(result, PathBuf::from("/build/deps/lib.o"));
    }

    #[test]
    fn test_generate_with_clang() {
        let deps_dir = Path::new("/build/deps");
        let source = SourceFilePath(PathBuf::from("src/hello.cpp"));

        let result = ObjectFileNaming::generate(deps_dir, &source, CompilerKind::Clang);

        assert_eq!(result, PathBuf::from("/build/deps/hello.o"));
    }

    #[test]
    fn test_generate_with_complex_path() {
        let deps_dir = Path::new("/project/target/deps");
        let source = SourceFilePath(PathBuf::from("src/submodule/feature.rs"));

        let result = ObjectFileNaming::generate(deps_dir, &source, CompilerKind::Msvc);

        assert_eq!(result, PathBuf::from("/project/target/deps/feature.obj"));
    }
}
