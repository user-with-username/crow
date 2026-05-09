#[cfg(test)]
mod tests {
    use crow_utils::hooks::run_hooks;
    use tempfile::TempDir;

    #[test]
    fn test_run_hooks_success() {
        let temp_dir = TempDir::new().unwrap();
        
        #[cfg(windows)]
        let hooks = vec!["cmd /c echo Hello".to_string()];
        
        #[cfg(unix)]
        let hooks = vec!["echo Hello".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_failure() {
        let temp_dir = TempDir::new().unwrap();
        
        #[cfg(windows)]
        let hooks = vec!["cmd /c exit 1".to_string()];
        
        #[cfg(unix)]
        let hooks = vec!["sh -c 'exit 1'".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("hook failed") || err_msg.contains("failed"));
    }

    #[test]
    fn test_run_hooks_multiple() {
        let temp_dir = TempDir::new().unwrap();
        
        #[cfg(windows)]
        let hooks = vec![
            "cmd /c echo First".to_string(),
            "cmd /c echo Second".to_string(),
        ];
        
        #[cfg(unix)]
        let hooks = vec![
            "echo First".to_string(),
            "echo Second".to_string(),
        ];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_invalid_command() {
        let temp_dir = TempDir::new().unwrap();
        let hooks = vec!["nonexistent_command_12345".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to execute"));
    }

    #[test]
    fn test_run_hooks_empty_hook() {
        let temp_dir = TempDir::new().unwrap();
        let hooks = vec!["".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty hook command"));
    }

    #[test]
    fn test_run_hooks_whitespace_only() {
        let temp_dir = TempDir::new().unwrap();
        let hooks = vec!["   ".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty hook command"));
    }

    #[test]
    fn test_run_hooks_output_handling() {
        let temp_dir = TempDir::new().unwrap();
        
        #[cfg(windows)]
        let hooks = vec![
            "cmd /c echo stdout message".to_string(),
            "cmd /c echo stderr message 1>&2".to_string(),
        ];
        
        #[cfg(unix)]
        let hooks = vec![
            "sh -c 'echo stdout message'".to_string(),
            "sh -c 'echo stderr message >&2'".to_string(),
        ];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_with_quotes() {
        let temp_dir = TempDir::new().unwrap();
        
        #[cfg(windows)]
        let hooks = vec!["cmd /c echo \"Hello World\"".to_string()];
        
        #[cfg(unix)]
        let hooks = vec!["echo \"Hello World\"".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_invalid_quoting() {
        let temp_dir = TempDir::new().unwrap();
        let hooks = vec!["echo \"unclosed quote".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to parse"));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_hooks_with_pipe() {
        let temp_dir = TempDir::new().unwrap();
        let hooks = vec!["echo 'hello world' | grep hello".to_string()];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_hooks_large_output() {
        let temp_dir = TempDir::new().unwrap();
        
        #[cfg(windows)]
        let hooks = vec![
            "cmd /c for /l %i in (1,1,100) do echo Line %i".to_string()
        ];
        
        #[cfg(unix)]
        let hooks = vec![
            "sh -c 'for i in $(seq 1 100); do echo Line $i; done'".to_string()
        ];
        
        let result = run_hooks(temp_dir.path(), &hooks);
        assert!(result.is_ok());
    }
}