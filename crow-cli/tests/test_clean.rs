#[cfg(test)]
mod tests {
    use crow_cli::commands::clean::{CleanArgs, CleanCommand};
    use tempfile::tempdir;

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
}
