#[cfg(test)]
mod tests {
    use crow_core::builder::paths::SourceFilePath;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn test_as_path() {
        let path = SourceFilePath(PathBuf::from("/home/user/file.txt"));
        assert_eq!(path.as_path(), Path::new("/home/user/file.txt"));
    }

    #[test]
    fn test_stem_with_extension() {
        let path = SourceFilePath(PathBuf::from("document.rs"));
        assert_eq!(path.stem(), Some("document".to_string()));
    }

    #[test]
    fn test_stem_without_extension() {
        let path = SourceFilePath(PathBuf::from("README"));
        assert_eq!(path.stem(), Some("README".to_string()));
    }

    #[test]
    fn test_stem_with_multiple_extensions() {
        let path = SourceFilePath(PathBuf::from("archive.tar.gz"));
        assert_eq!(path.stem(), Some("archive.tar".to_string()));
    }

    #[test]
    fn test_stem_hidden_file() {
        let path = SourceFilePath(PathBuf::from(".bashrc"));
        assert_eq!(path.stem(), Some(".bashrc".to_string()));
    }

    #[test]
    fn test_clone() {
        let path = SourceFilePath(PathBuf::from("test.txt"));
        let cloned = path.clone();
        assert_eq!(path.as_path(), cloned.as_path());
    }

    #[test]
    fn test_debug_format() {
        let path = SourceFilePath(PathBuf::from("file.txt"));
        let debug_str = format!("{:?}", path);
        assert!(debug_str.contains("file.txt"));
    }
}
