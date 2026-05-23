#[cfg(test)]
mod tests {
    use crow_core::builder::incremental::hash_files;
    use crow_core::builder::incremental::hash_source;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_hash_source_basic() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("main.c");

        let mut src_file = File::create(&src_path)?;
        src_file.write_all(b"int main() { return 0; }")?;

        let compiler = "gcc";
        let flags = vec!["-O2".to_string(), "-Wall".to_string()];
        let headers = vec![];

        let hash = hash_source(&src_path, &headers, compiler, &flags)?;

        // hash should be 64 hex chars
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        Ok(())
    }

    #[test]
    fn test_hash_source_same_content_same_hash() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path1 = dir.path().join("file1.c");
        let src_path2 = dir.path().join("file2.c");

        let content = b"int main() { return 0; }";
        File::create(&src_path1)?.write_all(content)?;
        File::create(&src_path2)?.write_all(content)?;

        let compiler = "gcc";
        let flags = vec!["-O2".to_string()];

        let hash1 = hash_source(&src_path1, &[], compiler, &flags)?;
        let hash2 = hash_source(&src_path2, &[], compiler, &flags)?;

        assert_eq!(hash1, hash2);

        Ok(())
    }

    #[test]
    fn test_hash_source_different_content_different_hash() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("main.c");

        File::create(&src_path)?.write_all(b"int main() { return 0; }")?;
        let hash1 = hash_source(&src_path, &[], "gcc", &["-O2".to_string()])?;

        File::create(&src_path)?.write_all(b"int main() { return 1; }")?;
        let hash2 = hash_source(&src_path, &[], "gcc", &["-O2".to_string()])?;

        assert_ne!(hash1, hash2);

        Ok(())
    }

    #[test]
    fn test_hash_source_different_compiler_different_hash() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("main.c");
        File::create(&src_path)?.write_all(b"int main() { return 0; }")?;

        let hash_gcc = hash_source(&src_path, &[], "gcc", &["-O2".to_string()])?;
        let hash_clang = hash_source(&src_path, &[], "clang", &["-O2".to_string()])?;

        assert_ne!(hash_gcc, hash_clang);

        Ok(())
    }

    #[test]
    fn test_hash_source_different_flags_different_hash() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("main.c");
        File::create(&src_path)?.write_all(b"int main() { return 0; }")?;

        let hash_o2 = hash_source(&src_path, &[], "gcc", &["-O2".to_string()])?;
        let hash_o3 = hash_source(&src_path, &[], "gcc", &["-O3".to_string()])?;

        assert_ne!(hash_o2, hash_o3);

        Ok(())
    }

    #[test]
    fn test_hash_source_with_headers() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("main.c");
        let header_path = dir.path().join("utils.h");

        File::create(&src_path)?.write_all(b"#include \"utils.h\"\nint main() { return 0; }")?;
        File::create(&header_path)?.write_all(b"void helper();")?;

        let hash_with_header = hash_source(&src_path, &[header_path.clone()], "gcc", &[])?;
        let hash_without_header = hash_source(&src_path, &[], "gcc", &[])?;

        assert_ne!(hash_with_header, hash_without_header);

        Ok(())
    }

    #[test]
    fn test_hash_source_headers_order_independent() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("main.c");
        let header1 = dir.path().join("a.h");
        let header2 = dir.path().join("b.h");

        File::create(&src_path)?.write_all(b"int main() { return 0; }")?;
        File::create(&header1)?.write_all(b"// header A")?;
        File::create(&header2)?.write_all(b"// header B")?;

        let hash_order1 = hash_source(&src_path, &[header1.clone(), header2.clone()], "gcc", &[])?;
        let hash_order2 = hash_source(&src_path, &[header2, header1], "gcc", &[])?;

        assert_eq!(hash_order1, hash_order2);

        Ok(())
    }

    #[test]
    fn test_hash_source_missing_header() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("main.c");
        let missing_header = dir.path().join("missing.h");

        File::create(&src_path)?.write_all(b"int main() { return 0; }")?;

        let hash = hash_source(&src_path, &[missing_header.clone()], "gcc", &[])?;

        assert_eq!(hash.len(), 64);

        let hash2 = hash_source(&src_path, &[missing_header], "gcc", &[])?;
        assert_eq!(hash, hash2);

        Ok(())
    }

    #[test]
    fn test_hash_source_empty_file() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("empty.c");
        File::create(&src_path)?;

        let hash = hash_source(&src_path, &[], "gcc", &[])?;

        assert_eq!(hash.len(), 64);

        Ok(())
    }

    #[test]
    fn test_hash_source_large_file() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let src_path = dir.path().join("large.c");

        let mut file = File::create(&src_path)?;
        let data = vec![b'A'; 1024 * 1024];
        file.write_all(&data)?;

        let hash = hash_source(&src_path, &[], "gcc", &[])?;

        assert_eq!(hash.len(), 64);

        Ok(())
    }

    #[test]
    fn test_hash_files_basic() -> anyhowed::Result<()> {
        let dir = tempdir()?;
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");

        File::create(&file1)?.write_all(b"content1")?;
        File::create(&file2)?.write_all(b"content2")?;

        let hash = hash_files(&[file1, file2])?;

        assert_eq!(hash.len(), 64);

        Ok(())
    }
}
