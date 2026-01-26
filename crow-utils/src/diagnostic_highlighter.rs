use regex::Regex;

pub struct DiagnosticHighlighter;

impl DiagnosticHighlighter {
    pub fn colorize(line: &str) -> String {
        let mut s = line.to_string();

        let patterns = [
            (
                r"^(?P<prefix>[\w\.\-_/\\]+(?:\.\w+)?:\d+(?::\d+)?:)\s*(?P<type>error|fatal error|warning|note):",
                true,
            ),
            (
                r"^(?P<prefix>[\w\.\-_/\\]+\.\w+\(\d+\) : )\s*(?P<type>error|fatal error|warning)\s*(?P<code>C\d+):",
                false,
            ),
        ];

        for (pattern, simple) in patterns {
            if let Ok(re) = Regex::new(pattern) {
                s = re
                    .replace_all(&s, |caps: &regex::Captures| {
                        let prefix = caps.name("prefix").unwrap().as_str();
                        let r#type = caps.name("type").unwrap().as_str();
                        let color = match r#type {
                            "error" | "fatal error" => "\x1b[91m",
                            "warning" => "\x1b[93m",
                            "note" => "\x1b[94m",
                            _ => "\x1b[0m",
                        };

                        if simple {
                            format!("{prefix} {color}{type}\x1b[0m:")
                        } else {
                            let code = caps.name("code").unwrap().as_str();
                            format!("{prefix} {color}{type}\x1b[0m {code}:")
                        }
                    })
                    .to_string();
                break;
            }
        }

        s
    }
}
