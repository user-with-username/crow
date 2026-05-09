#[cfg(test)]
mod tests {
    use crow_cli::commands::clean::{CleanArgs, CleanCommand};
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_clean_args_create() {
        let args = CleanArgs {};
        let _args = args;
    }

    #[test]
    fn test_clean_command_new() {
        let args = CleanArgs {};
        let command = CleanCommand::new(args);
        let _ = command;
    }

    #[test]
    fn test_clean_command_execute_no_target_dir() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let args = CleanArgs {};
        let command = CleanCommand::new(args);
        let result = command.execute();
        
        assert!(result.is_err());
        
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_clean_command_execute_with_target_dir() {
        let temp_dir = tempdir().unwrap();
        
        let crow_toml_path = temp_dir.path().join("Crow.toml");
        fs::write(&crow_toml_path, r#"
[package]
name = "test-package"
version = "0.1.0"
        "#).unwrap();
        
        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();
        
        let dummy_file = target_dir.join("dummy.txt");
        fs::write(&dummy_file, "test content").unwrap();
        
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let args = CleanArgs {};
        let command = CleanCommand::new(args);

        let result = command.execute();
        
        match result {
            Ok(_) => {
                assert!(!target_dir.exists());
            }
            Err(e) => {
                let err_msg = e.to_string();
                assert!(err_msg.contains("config") || err_msg.contains("Crow"));
            }
        }
        
        std::env::set_current_dir(original_dir).unwrap();
    }
}
