#[cfg(test)]
mod tests {
    use crow_cli::commands::init::{InitArgs, InitCommand};
    use tempfile::tempdir;

    #[test]
    fn test_init_args_default() {
        let args = InitArgs {
            quiet: false,
        };
        
        assert!(!args.quiet);
    }

    #[test]
    fn test_init_args_quiet() {
        let args = InitArgs {
            quiet: true,
        };
        
        assert!(args.quiet);
    }

    #[test]
    fn test_init_command_new() {
        let args = InitArgs {
            quiet: false,        };
        
        let command = InitCommand::new(args);
        let _ = command;
    }

    #[test]
    fn test_init_command_quiet_mode_no_overwrite_prompt() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let args = InitArgs {
            quiet: true,
        };
        
        let command = InitCommand::new(args);
        let result = command.execute();
        
        assert!(result.is_ok());
        
        assert!(temp_dir.path().join("crow.toml").exists());
        assert!(temp_dir.path().join("src/main.cpp").exists());
        
        std::env::set_current_dir(original_dir).unwrap();
    }
}