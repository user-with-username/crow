// useless (just for api checking)

#[cfg(test)]
mod tests {
    use crow_utils::Logger;

    #[test]
    fn test_new_logger_not_quiet_by_default() {
        let logger = Logger::new();
        assert!(!logger.is_quiet());
    }

    #[test]
    fn test_status_silent_when_quiet() {
        let logger = Logger { quiet: true };
        logger.status("TEST", "message");
    }

    #[test]
    fn test_warning_always_outputs() {
        let logger = Logger { quiet: true };
        logger.warning("test warning");
    }

    #[test]
    fn test_error_always_outputs() {
        let logger = Logger { quiet: true };
        logger.error("test error");
    }
}