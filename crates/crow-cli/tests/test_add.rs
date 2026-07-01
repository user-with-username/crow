#[cfg(test)]
mod tests {
    use crow_cli::commands::add::{AddArgs, AddCommand};

    #[test]
    fn test_add_args_default() {
        let args = AddArgs {
            package: "test_pkg".to_string(),
            version: None,
            registry: None,
            dev: false,
        };

        assert_eq!(args.package, "test_pkg");
        assert!(args.version.is_none());
        assert!(args.registry.is_none());
    }

    #[test]
    fn test_add_args_with_version() {
        let args = AddArgs {
            package: "test_pkg".to_string(),
            version: Some("1.0.0".to_string()),
            registry: None,
            dev: false,
        };

        assert_eq!(args.package, "test_pkg");
        assert_eq!(args.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_add_command_new() {
        let args = AddArgs {
            package: "test_pkg".to_string(),
            version: None,
            registry: None,
            dev: false,
        };

        let command = AddCommand::new(args);
        let _ = command;
    }
}
