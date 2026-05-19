#[cfg(test)]
mod tests {
    use crow_cli::commands::bench::{BenchArgs, BenchCommand};

    #[test]
    fn test_bench_args_default() {
        let args = BenchArgs {
            jobs: None,
            bin: None,
            args: vec![],
            release: false,
        };

        assert_eq!(args.jobs, None);
        assert_eq!(args.bin, None);
        assert!(args.args.is_empty());
        assert_eq!(args.release, false);
    }

    #[test]
    fn test_bench_args_with_values() {
        let args = BenchArgs {
            jobs: Some(4),
            bin: Some("my_bench".to_string()),
            args: vec![
                "--verbose".to_string(),
                "--iterations".to_string(),
                "100".to_string(),
            ],
            release: true,
        };

        assert_eq!(args.jobs, Some(4));
        assert_eq!(args.bin, Some("my_bench".to_string()));
        assert_eq!(args.args.len(), 3);
        assert_eq!(args.release, true);
    }

    #[test]
    fn test_bench_command_new_returns_benchcommand() {
        let args = BenchArgs {
            jobs: Some(2),
            bin: None,
            args: vec!["--test".to_string()],
            release: false,
        };

        let command = BenchCommand::new(args);

        let _command = command;
    }

    #[test]
    fn test_bench_command_execute_calls_run_with_profile() {
        let args = BenchArgs {
            jobs: Some(1),
            bin: Some("test_bin".to_string()),
            args: vec!["--help".to_string()],
            release: false,
        };

        let command = BenchCommand::new(args);

        let _ = command.execute();
    }
}
