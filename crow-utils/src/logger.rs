pub struct Logger {
    pub quiet: bool,
}

impl Logger {
    pub fn new() -> Self {
        let quiet = std::env::args().any(|arg| arg == "-q" || arg == "--quiet");
        Self { quiet }
    }

    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    pub fn error(&self, msg: &str) {
        eprintln!("\x1b[91merror\x1b[0m: {}", msg);
    }

    pub fn status(&self, status: &str, msg: &str) {
        if self.quiet {
            return;
        }
        println!("\x1b[1;92m{:>12}\x1b[0m {}", status, msg);
    }

    pub fn warning(&self, msg: &str) {
        eprintln!("\x1b[93mwarning\x1b[0m: {}", msg);
    }
}

pub fn get_logger() -> Logger {
    Logger::new()
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let logger = $crate::logger::get_logger();
        logger.error(&format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! status {
    ($status:literal, $($arg:tt)*) => {{
        let logger = $crate::logger::get_logger();
        logger.status($status, &format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        let logger = $crate::logger::get_logger();
        logger.warning(&format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! show_output {
    ($output:expr, $project:expr) => {
        use $crate::DiagnosticHighlighter;

        let is_msvc = $project.compiler_kind().is_msvc();

        String::from_utf8_lossy(&$output.stderr)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .for_each(|line| eprintln!("{}", DiagnosticHighlighter::colorize(line)));

        String::from_utf8_lossy(&$output.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter(|line| {
                if is_msvc {
                    let upper = line.trim().to_uppercase();
                    upper.contains("WARNING") || upper.contains("ERROR")
                } else {
                    true
                }
            })
            .for_each(|line| eprintln!("{}", DiagnosticHighlighter::colorize(line)));
    };
}
