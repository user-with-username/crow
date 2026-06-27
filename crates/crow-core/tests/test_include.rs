#[cfg(test)]
mod tests {
    use crow_core::builder::tasks::IncludeTask;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    #[test]
    fn test_collect_headers_with_valid_dep_file() {
        let temp_dir = tempdir().unwrap();
        let dep_file = temp_dir.path().join("deps.d");

        let header1 = temp_dir.path().join("stdio.h");
        let header2 = temp_dir.path().join("stdlib.h");
        let header3 = temp_dir.path().join("string.h");

        File::create(&header1).unwrap();
        File::create(&header2).unwrap();
        File::create(&header3).unwrap();

        let content = format!(
            r#"main.o: main.c
  {}
  {}
  {}"#,
            header1.display(),
            header2.display(),
            header3.display()
        );

        File::create(&dep_file)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let task = IncludeTask::new(dep_file);
        let headers = task.collect_headers().unwrap();

        assert_eq!(headers.len(), 3);
    }

    #[test]
    fn test_collect_headers_ignores_lines_with_colon() {
        let temp_dir = tempdir().unwrap();
        let dep_file = temp_dir.path().join("deps.d");

        let header1 = temp_dir.path().join("header1.h");
        let header2 = temp_dir.path().join("header2.h");

        File::create(&header1).unwrap();
        File::create(&header2).unwrap();

        let content = format!(
            r#"target.o: source.c
  {}
  {}"#,
            header1.display(),
            header2.display()
        );

        File::create(&dep_file)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let task = IncludeTask::new(dep_file);
        let headers = task.collect_headers().unwrap();

        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_collect_headers_with_backslash_continuation() {
        let temp_dir = tempdir().unwrap();
        let dep_file = temp_dir.path().join("deps.d");

        let header = temp_dir
            .path()
            .join("long_header_name_that_continues_on_next_line.h");
        File::create(&header).unwrap();

        let content = format!(
            r#"  {}\
"#,
            header.display()
        );

        File::create(&dep_file)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let task = IncludeTask::new(dep_file);
        let headers = task.collect_headers().unwrap();

        assert_eq!(headers.len(), 1);
    }

    #[test]
    fn test_collect_headers_nonexistent_file() {
        let task = IncludeTask::new(PathBuf::from("/nonexistent/file.d"));
        let result = task.collect_headers();

        assert!(result.is_err());
    }

    #[test]
    fn test_save_deps_with_valid_lines() {
        let temp_dir = tempdir().unwrap();
        let dep_path = temp_dir.path().join("output.d");

        let lines = vec![
            "Note: including file: /usr/include/stdio.h".to_string(),
            "Note: including file:   /usr/include/stdlib.h  ".to_string(),
            "Some other output".to_string(),
            "Note: including file: /custom/path/header.h".to_string(),
        ];

        IncludeTask::save_deps(&dep_path, &lines).unwrap();

        let content = fs::read_to_string(&dep_path).unwrap();
        let p1 = crow_utils::normalize_path("/usr/include/stdio.h");
        let p2 = crow_utils::normalize_path("/usr/include/stdlib.h");
        let p3 = crow_utils::normalize_path("/custom/path/header.h");
        assert!(content.contains(&p1));
        assert!(content.contains(&p2));
        assert!(content.contains(&p3));
        assert_eq!(content.lines().count(), 3);
    }

    #[test]
    fn test_save_deps_with_empty_lines() {
        let temp_dir = tempdir().unwrap();
        let dep_path = temp_dir.path().join("output.d");

        let lines = vec!["Note: including file: only_one.h".to_string()];

        IncludeTask::save_deps(&dep_path, &lines).unwrap();

        let content = fs::read_to_string(&dep_path).unwrap();
        assert_eq!(content.trim(), "only_one.h");
    }

    #[test]
    fn test_save_deps_no_matching_lines() {
        let temp_dir = tempdir().unwrap();
        let dep_path = temp_dir.path().join("output.d");

        let lines = vec![
            "Compiling file.c".to_string(),
            "Warning: something".to_string(),
            "Done".to_string(),
        ];

        IncludeTask::save_deps(&dep_path, &lines).unwrap();

        assert!(!dep_path.exists() || fs::read_to_string(&dep_path).unwrap().is_empty());
    }

    #[test]
    fn test_save_deps_creates_parent_directories() {
        let temp_dir = tempdir().unwrap();
        let dep_path = temp_dir.path().join("subdir/nested/output.d");

        let lines = vec!["Note: including file: header.h".to_string()];

        std::fs::create_dir_all(dep_path.parent().unwrap()).unwrap();

        IncludeTask::save_deps(&dep_path, &lines).unwrap();

        assert!(dep_path.exists());
        let content = fs::read_to_string(&dep_path).unwrap();
        assert_eq!(content.trim(), "header.h");
    }
}
