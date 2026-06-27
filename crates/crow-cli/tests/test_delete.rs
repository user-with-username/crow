#[cfg(test)]
mod tests {
    use crow_cli::commands::delete::{DeleteArgs, DeleteCommand};

    #[test]
    fn test_delete_args_default() {
        let args = DeleteArgs {
            package: "test-pkg".to_string(),
            version: None,
            registry: None,
            yes: false,
            dry_run: false,
        };

        assert_eq!(args.package, "test-pkg");
        assert_eq!(args.version, None);
        assert_eq!(args.registry, None);
        assert!(!args.yes);
        assert!(!args.dry_run);
    }

    #[test]
    fn test_delete_args_with_version() {
        let args = DeleteArgs {
            package: "serde".to_string(),
            version: Some("1.0.0".to_string()),
            registry: Some("https://custom-registry.com".to_string()),
            yes: true,
            dry_run: true,
        };

        assert_eq!(args.package, "serde");
        assert_eq!(args.version, Some("1.0.0".to_string()));
        assert_eq!(
            args.registry,
            Some("https://custom-registry.com".to_string())
        );
        assert!(args.yes);
        assert!(args.dry_run);
    }

    #[test]
    fn test_delete_command_new() {
        let args = DeleteArgs {
            package: "test".to_string(),
            version: None,
            registry: None,
            yes: false,
            dry_run: false,
        };

        let command = DeleteCommand::new(args);
        let _ = command;
    }

    #[test]
    fn test_delete_command_execute_dry_run() {
        let args = DeleteArgs {
            package: "test-package".to_string(),
            version: Some("1.0.0".to_string()),
            registry: None,
            yes: true,
            dry_run: true,
        };

        let command = DeleteCommand::new(args);
        let result = command.execute();

        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_command_without_confirmation_uses_yes_flag() {
        let args = DeleteArgs {
            package: "test-package".to_string(),
            version: None,
            registry: None,
            yes: true,
            dry_run: false,
        };

        let command = DeleteCommand::new(args);

        let result = command.execute();

        assert!(result.is_err());
    }

    #[test]
    fn test_delete_command_target_format_with_version() {
        let args = DeleteArgs {
            package: "tokio".to_string(),
            version: Some("1.0.0".to_string()),
            registry: None,
            yes: true,
            dry_run: true,
        };

        let command = DeleteCommand::new(args);
        let result = command.execute();

        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_command_target_format_without_version() {
        let args = DeleteArgs {
            package: "regex".to_string(),
            version: None,
            registry: None,
            yes: true,
            dry_run: true,
        };

        let command = DeleteCommand::new(args);
        let result = command.execute();

        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_command_custom_registry_url() {
        let custom_registry = "https://my-registry.com/api";

        let args = DeleteArgs {
            package: "custom-pkg".to_string(),
            version: Some("2.0.0".to_string()),
            registry: Some(custom_registry.to_string()),
            yes: true,
            dry_run: true,
        };

        let command = DeleteCommand::new(args);
        let result = command.execute();

        assert!(result.is_ok());
    }
}
