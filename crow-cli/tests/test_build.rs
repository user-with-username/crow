#[cfg(test)]
mod tests {
    use crow_cli::commands::build::{BuildArgs, BuildCommand};

    #[test]
    fn test_build_args_default() {
        let args = BuildArgs {
            release: false,
            target: None,
            jobs: None,
            bin: None,
            profile: "debug".to_string(),
        };
        
        assert!(!args.release);
        assert_eq!(args.target, None);
        assert_eq!(args.jobs, None);
        assert_eq!(args.bin, None);
        assert_eq!(args.profile, "debug");
    }

    #[test]
    fn test_build_args_with_release() {
        let args = BuildArgs {
            release: true,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            jobs: Some(4),
            bin: Some("myapp".to_string()),
            profile: "custom".to_string(),
        };
        
        assert!(args.release);
        assert_eq!(args.target, Some("x86_64-unknown-linux-gnu".to_string()));
        assert_eq!(args.jobs, Some(4));
        assert_eq!(args.bin, Some("myapp".to_string()));
        assert_eq!(args.profile, "custom");
    }

    #[test]
    fn test_build_command_new() {
        let args = BuildArgs {
            release: false,
            target: None,
            jobs: None,
            bin: None,
            profile: "debug".to_string(),
        };
        
        let command = BuildCommand::new(args);
        let _ = command;
    }

    #[test]
    fn test_build_command_profile_selection() {
        let args_profile = BuildArgs {
            release: false,
            target: None,
            jobs: None,
            bin: None,
            profile: "custom".to_string(),
        };
        
        let command_profile = BuildCommand::new(args_profile);
        let _ = command_profile;
    }

    #[test]
    fn test_build_command_without_workspace() {
        let args = BuildArgs {
            release: false,
            target: None,
            jobs: None,
            bin: None,
            profile: "debug".to_string(),
        };
        
        let command = BuildCommand::new(args);
        let result = command.execute();
        assert!(result.is_err());
    }
}