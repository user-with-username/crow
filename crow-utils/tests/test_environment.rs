#[cfg(test)]
mod tests {
    use crow_utils::environment::Environment;
    use std::path::PathBuf;
    use temp_env::with_var;

    #[test]
    fn test_target_dir_with_env_var() {
        with_var("CROW_TARGET_DIR", Some("custom_target"), || {
            let target_dir = Environment::target_dir();
            assert_eq!(target_dir, PathBuf::from("custom_target"));
        });
    }

    #[test]
    fn test_target_dir_without_env_var() {
        with_var("CROW_TARGET_DIR", None::<&str>, || {
            let target_dir = Environment::target_dir();
            assert_eq!(target_dir, PathBuf::from("target"));
        });
    }

    #[test]
    fn test_target_dir_with_empty_env_var() {
        with_var("CROW_TARGET_DIR", Some(""), || {
            let target_dir = Environment::target_dir();
            assert_eq!(target_dir, PathBuf::from(""));
        });
    }

    #[test]
    fn test_github_token_with_env_var() {
        with_var("GITHUB_TOKEN", Some("test_token_123"), || {
            let token = Environment::github_token();
            assert!(token.is_ok());
            assert_eq!(token.unwrap(), "test_token_123");
        });
    }

    #[test]
    fn test_github_token_without_env_var() {
        with_var("GITHUB_TOKEN", None::<&str>, || {
            let token = Environment::github_token();
            assert!(token.is_err());

            let err = token.unwrap_err();
            let err_msg = format!("{}", err);
            assert!(err_msg.contains("GITHUB_TOKEN"));
            assert!(err_msg.contains("environment variable is required"));
            assert!(err_msg.contains("https://github.com/settings/tokens"));
        });
    }

    #[test]
    fn test_github_token_with_empty_env_var() {
        with_var("GITHUB_TOKEN", Some(""), || {
            let token = Environment::github_token();
            assert!(token.is_ok());
            assert_eq!(token.unwrap(), "");
        });
    }
}
