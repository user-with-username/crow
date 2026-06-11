#[cfg(test)]
mod tests {
    use crow_cli::commands::check::{CheckArgs, CheckCommand};

    #[test]
    fn test_check_args_default() {
        let args = CheckArgs {
            release: false,
            jobs: None,
            bin: None,
            profile: "debug".to_string(),
            target: None,
        };

        assert!(!args.release);
        assert_eq!(args.jobs, None);
        assert_eq!(args.bin, None);
        assert_eq!(args.profile, "debug");
    }

    #[test]
    fn test_check_args_with_release() {
        let args = CheckArgs {
            release: true,
            jobs: Some(4),
            bin: Some("myapp".to_string()),
            profile: "release".to_string(),
            target: None,
        };

        assert!(args.release);
        assert_eq!(args.jobs, Some(4));
        assert_eq!(args.bin, Some("myapp".to_string()));
        assert_eq!(args.profile, "release");
    }

    #[test]
    fn test_check_command_new() {
        let args = CheckArgs {
            release: false,
            jobs: None,
            bin: None,
            profile: "debug".to_string(),
            target: None,
        };

        let command = CheckCommand::new(args);
        let _ = command;
    }

    #[test]
    fn test_check_command_profile_selection() {
        let args = CheckArgs {
            release: false,
            jobs: None,
            bin: None,
            profile: "test".to_string(),
            target: None,
        };

        let command = CheckCommand::new(args);
        let _ = command;
    }

    #[test]
    fn test_check_command_without_workspace() {
        let args = CheckArgs {
            release: false,
            jobs: None,
            bin: None,
            profile: "debug".to_string(),
            target: None,
        };

        let command = CheckCommand::new(args);
        let result = command.execute();
        assert!(result.is_err());
    }
}
