#[cfg(test)]
mod tests {
    use crow_cli::commands::publish::{PublishArgs, PublishCommand};
    use tempfile::tempdir;

    #[test]
    fn test_publish_args_default() {
        let args = PublishArgs {
            registry: None,
            dry_run: false,
            version: None,
            target: None,
        };

        assert_eq!(args.registry, None);
        assert!(!args.dry_run);
        assert_eq!(args.version, None);
    }

    #[test]
    fn test_publish_args_with_values() {
        let args = PublishArgs {
            registry: Some("https://custom-registry.com".to_string()),
            dry_run: true,
            version: Some("1.2.3".to_string()),
            target: None,
        };

        assert_eq!(
            args.registry,
            Some("https://custom-registry.com".to_string())
        );
        assert!(args.dry_run);
        assert_eq!(args.version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_publish_command_new() {
        let args = PublishArgs {
            registry: None,
            dry_run: false,
            version: None,
            target: None,
        };

        let command = PublishCommand::new(args);
        let _ = command;
    }

    #[test]
    fn test_publish_command_no_crow_toml() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let args = PublishArgs {
            registry: None,
            dry_run: false,
            version: None,
            target: None,
        };

        let command = PublishCommand::new(args);
        let result = command.execute();

        assert!(result.is_err());

        std::env::set_current_dir(original_dir).unwrap();
    }
}
