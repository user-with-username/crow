use regex::Regex;

pub struct DiagnosticHighlighter;

impl DiagnosticHighlighter {
    pub fn colorize(line: &str) -> String {
        Regex::new(r"(?i)(error|fatal error|warning|note)")
            .unwrap()
            .replace_all(line, |caps: &regex::Captures| {
                let r#type = caps.get(0).unwrap().as_str();
                let color = match r#type.to_lowercase().as_str() {
                    "error" | "fatal error" => "\x1b[91m",
                    "warning" => "\x1b[93m",
                    "note" => "\x1b[94m",
                    _ => "\x1b[0m",
                };
                format!("{color}{type}\x1b[0m")
            })
            .to_string()
    }
}
