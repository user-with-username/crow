#[cfg(test)]
mod tests {
    use crow_utils::DiagnosticHighlighter;

    #[test]
    fn test_colorize_error() {
        let line = "error: could not compile package";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[91merror\x1b[0m"));
        assert_eq!(result, "\x1b[91merror\x1b[0m: could not compile package");
    }

    #[test]
    fn test_colorize_fatal_error() {
        let line = "fatal error: out of memory";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[91mfatal error\x1b[0m"));
        assert_eq!(result, "\x1b[91mfatal error\x1b[0m: out of memory");
    }

    #[test]
    fn test_colorize_warning() {
        let line = "warning: unused variable";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[93mwarning\x1b[0m"));
        assert_eq!(result, "\x1b[93mwarning\x1b[0m: unused variable");
    }

    #[test]
    fn test_colorize_note() {
        let line = "note: this function is deprecated";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[94mnote\x1b[0m"));
        assert_eq!(result, "\x1b[94mnote\x1b[0m: this function is deprecated");
    }

    #[test]
    fn test_colorize_case_insensitive_error() {
        let line = "ERROR: authentication failed";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[91mERROR\x1b[0m"));
        assert_eq!(result, "\x1b[91mERROR\x1b[0m: authentication failed");
    }

    #[test]
    fn test_colorize_case_insensitive_warning() {
        let line = "WARNING: deprecated feature";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[93mWARNING\x1b[0m"));
        assert_eq!(result, "\x1b[93mWARNING\x1b[0m: deprecated feature");
    }

    #[test]
    fn test_colorize_no_keyword() {
        let line = "Compiling myproject v0.1.0";
        let result = DiagnosticHighlighter::colorize(line);

        assert_eq!(result, "Compiling myproject v0.1.0");
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_colorize_multiple_keywords() {
        let line = "error: something failed\nwarning: but we continued\nnote: just so you know";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[91merror\x1b[0m"));
        assert!(result.contains("\x1b[93mwarning\x1b[0m"));
        assert!(result.contains("\x1b[94mnote\x1b[0m"));
        assert!(result.contains("\x1b[91merror\x1b[0m: something failed\n\x1b[93mwarning\x1b[0m: but we continued\n\x1b[94mnote\x1b[0m: just so you know"));
    }

    #[test]
    fn test_colorize_with_mixed_case() {
        let line = "Error: mixed case test";
        let result = DiagnosticHighlighter::colorize(line);

        assert!(result.contains("\x1b[91mError\x1b[0m"));
        assert_eq!(result, "\x1b[91mError\x1b[0m: mixed case test");
    }

    #[test]
    fn test_colorize_with_special_characters() {
        let line = "error: failed to parse 'invalid\"string'";
        let result = DiagnosticHighlighter::colorize(line);

        assert_eq!(
            result,
            "\x1b[91merror\x1b[0m: failed to parse 'invalid\"string'"
        );
    }

    #[test]
    fn test_colorize_empty_string() {
        let line = "";
        let result = DiagnosticHighlighter::colorize(line);

        assert_eq!(result, "");
    }

    #[test]
    fn test_colorize_very_long_line() {
        let long_error = "error: ".to_owned() + &"a".repeat(1000);
        let result = DiagnosticHighlighter::colorize(&long_error);

        let expected = "\x1b[91merror\x1b[0m: ".to_owned() + &"a".repeat(1000);
        assert_eq!(result, expected);
    }
}
