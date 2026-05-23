use regex::Regex;
use std::sync::OnceLock;

pub struct DiagnosticHighlighter;

impl DiagnosticHighlighter {
    pub fn colorize(line: &str) -> String {
        static DIAGNOSTIC_PATTERN: OnceLock<Regex> = OnceLock::new();
        let re = DIAGNOSTIC_PATTERN.get_or_init(Self::create_diagnostic_pattern);
        re.replace_all(line, Self::colorize_match).into_owned()
    }

    fn create_diagnostic_pattern() -> Regex {
        Regex::new(concat!(
            r"(?i)",
            r"(?<gcc>\b(?:fatal error|error|warning|note):)|",
            r"(?<msvc>\b(?:fatal error|error|warning)\s+[A-Z]+\d+)|",
            r"(?<msvc_note>^\s*note:)"
        )).unwrap()
    }

    fn colorize_match(caps: &regex::Captures) -> String {
        if let Some(mat) = caps.name("gcc") {
            Self::colorize_gcc(mat.as_str())
        } else if let Some(mat) = caps.name("msvc") {
            Self::colorize_msvc(mat.as_str())
        } else if let Some(mat) = caps.name("msvc_note") {
            Self::colorize_msvc_note(mat.as_str())
        } else {
            caps.get(0).unwrap().as_str().to_string()
        }
    }

    fn colorize_gcc(text: &str) -> String {
        let clean_word = text.trim_end_matches(':').to_lowercase();
        let color = Self::get_color(&clean_word);
        format!("{}{}\x1b[0m:", color, &text[..text.len() - 1])
    }

    fn colorize_msvc(text: &str) -> String {
        let clean_word = text.split_whitespace().next().unwrap_or("").to_lowercase();
        let color = Self::get_color(&clean_word);
        format!("{}{}\x1b[0m", color, text)
    }

    fn colorize_msvc_note(text: &str) -> String {
        let trimmed = text.trim();
        let spaces = &text[..text.len() - trimmed.len()];
        format!("{spaces}\x1b[94m{}\x1b[0m:", &trimmed[..trimmed.len() - 1])
    }

    fn get_color(text_type: &str) -> &'static str {
        match text_type {
            "error" | "fatal error" => "\x1b[91m",
            "warning" => "\x1b[93m",
            "note" => "\x1b[94m",
            _ => "\x1b[0m",
        }
    }
}