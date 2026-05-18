#[cfg(test)]
mod tests {
    use crow_utils::files::{escape, write_to};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_escape_empty_string() {
        assert_eq!(escape(""), "\"\"");
    }

    #[test]
    fn test_escape_no_special_chars() {
        assert_eq!(escape("normal_text"), "normal_text");
        assert_eq!(escape("hello_world"), "hello_world");
        assert_eq!(escape("path/to/file"), "path/to/file");
    }

    #[test]
    fn test_escape_with_space() {
        assert_eq!(escape("hello world"), "\"hello world\"");
        assert_eq!(escape("two  spaces"), "\"two  spaces\"");
    }

    #[test]
    fn test_escape_with_tab() {
        assert_eq!(escape("hello\tworld"), "\"hello\tworld\"");
    }

    #[test]
    fn test_escape_with_quotes() {
        assert_eq!(escape("say \"hello\""), "\"say \"\"hello\"\"\"");
        assert_eq!(escape("\"quoted\""), "\"\"\"quoted\"\"\"");
    }

    #[test]
    fn test_escape_with_mixed_special_chars() {
        assert_eq!(
            escape("hello \"world\" test"),
            "\"hello \"\"world\"\" test\""
        );
        assert_eq!(escape("with space and\t tab"), "\"with space and\t tab\"");
    }

    #[test]
    fn test_write_to_basic() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args = ["arg1", "arg2", "arg3"];

        let path = write_to(&temp_dir, "test.args", &args, false)?;

        assert!(path.exists());
        let content = fs::read_to_string(&path)?;
        assert_eq!(content, "arg1\narg2\narg3\n");

        Ok(())
    }

    #[test]
    fn test_write_to_with_spaces() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args = ["hello world", "another arg", "simple"];

        let path = write_to(&temp_dir, "spaces.args", &args, false)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!(content, "\"hello world\"\n\"another arg\"\nsimple\n");

        Ok(())
    }

    #[test]
    fn test_write_to_with_backslashes_not_msvc() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args = [r"C:\windows\path", r"normal/path", r"with space\here"];

        let path = write_to(&temp_dir, "backslashes.args", &args, false)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!(
            content,
            "C:/windows/path\nnormal/path\n\"with space/here\"\n"
        );

        Ok(())
    }

    #[test]
    fn test_write_to_with_quotes() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args = ["say \"hello\"", "normal", "with \"quotes\" here"];

        let path = write_to(&temp_dir, "quotes.args", &args, false)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!(
            content,
            "\"say \"\"hello\"\"\"\nnormal\n\"with \"\"quotes\"\" here\"\n"
        );

        Ok(())
    }

    #[test]
    fn test_write_to_empty_args() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args: [&str; 0] = [];

        let path = write_to(&temp_dir, "empty.args", &args, false)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!(content, "");

        Ok(())
    }

    #[test]
    fn test_write_to_empty_string_arg() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args = ["", "normal"];

        let path = write_to(&temp_dir, "empty_string.args", &args, false)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!(content, "\"\"\nnormal\n");

        Ok(())
    }

    #[test]
    fn test_write_to_nested_filename() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args = ["arg1"];

        let path = write_to(&temp_dir, "subdir/nested.args", &args, false)?;

        assert!(path.exists());
        assert!(path.to_str().unwrap().contains("subdir"));
        let content = fs::read_to_string(&path)?;
        assert_eq!(content, "arg1\n");

        Ok(())
    }

    #[test]
    fn test_write_to_multiple_calls() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;

        let path1 = write_to(&temp_dir, "file1.args", &["first"], false)?;
        let path2 = write_to(&temp_dir, "file2.args", &["second"], false)?;

        assert!(path1.exists());
        assert!(path2.exists());
        assert_ne!(path1, path2);

        assert_eq!(fs::read_to_string(path1)?, "first\n");
        assert_eq!(fs::read_to_string(path2)?, "second\n");

        Ok(())
    }

    #[test]
    fn test_escape_is_used_correctly() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args = ["needs escaping here", "normal", "with\"quote"];

        let path = write_to(&temp_dir, "escape_test.args", &args, false)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!(
            content,
            "\"needs escaping here\"\nnormal\n\"with\"\"quote\"\n"
        );

        Ok(())
    }

    #[test]
    fn test_returns_correct_path() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let filename = "test.args";

        let path = write_to(&temp_dir, filename, &["arg"], false)?;

        let expected = temp_dir.path().join("args").join(filename);
        assert_eq!(path, expected);

        Ok(())
    }

    #[test]
    fn test_overwrites_existing_file() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let filename = "overwrite.args";

        write_to(&temp_dir, filename, &["first"], false)?;
        write_to(&temp_dir, filename, &["second"], false)?;

        let content = fs::read_to_string(temp_dir.path().join("args").join(filename))?;
        assert_eq!(content, "second\n");

        Ok(())
    }

    #[test]
    fn test_large_number_of_args() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let args: Vec<String> = (0..1000).map(|i| format!("arg{}", i)).collect();

        let path = write_to(&temp_dir, "large.args", &args, false)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!(content.lines().count(), 1000);
        assert!(content.starts_with("arg0\n"));
        assert!(content.contains("arg500\n"));
        assert!(content.ends_with("arg999\n"));

        Ok(())
    }
}
