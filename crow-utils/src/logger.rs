pub struct Logger {
    quiet: bool,
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
        if self.quiet { return; }
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