#[cfg(test)]
mod tests {
    use crow_cli::commands::metadata::{MetadataArgs, MetadataCommand};

    #[test]
    fn test_metadata_args_default() {
        let args = MetadataArgs {};

        let _ = args;
    }

    #[test]
    fn test_metadata_command_new_returns_metadata_command() {
        let args = MetadataArgs {};
        let command = MetadataCommand::new(args);

        let _ = command;
    }

    #[test]
    fn test_metadata_command_execute_returns_json_output() {
        let args = MetadataArgs {};
        let command = MetadataCommand::new(args);

        let result = command.execute();
        assert!(result.is_ok());
    }
}
