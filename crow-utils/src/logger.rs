// I'll rewrite it soon. now it works — it works

use std::sync::atomic::{AtomicBool, Ordering};

pub static QUIET_MODE: AtomicBool = AtomicBool::new(false);

pub const INDENT_LEVEL_1: &str = "  ";
pub const INDENT_LEVEL_2: &str = "    ";

pub struct Colors {
    pub green: &'static str,
    pub yellow: &'static str,
    pub cyan: &'static str,
    pub red: &'static str,
    pub dim: &'static str,
    pub reset: &'static str,
    pub bold: &'static str,
}

impl Colors {
    pub const fn new() -> Self {
        Colors {
            green: "\x1b[32m",
            yellow: "\x1b[33m",
            cyan: "\x1b[36m",
            red: "\x1b[31m",
            dim: "\x1b[2m",
            reset: "\x1b[0m",
            bold: "\x1b[1m",
        }
    }
}

pub enum LogLevel {
    Info,
    Warn,
    Error,
    Dim,
    Success,
    Bold,
    Custom(&'static str),
    CriticalError,
}

pub struct Logger {
    pub colors: Colors,
}

impl Logger {
    pub const fn new() -> Self {
        Logger {
            colors: Colors::new(),
        }
    }

    fn print_message(&self, level: LogLevel, indent_level: usize, message: &str) {
        if QUIET_MODE.load(Ordering::Relaxed)
            && !matches!(level, LogLevel::CriticalError | LogLevel::Error)
        {
            return;
        }

        let indent = match indent_level {
            1 => INDENT_LEVEL_1,
            2 => INDENT_LEVEL_2,
            _ => "",
        };

        let (base_color, text_prefix) = match level {
            LogLevel::Info => (self.colors.cyan, ""),
            LogLevel::Warn => (self.colors.yellow, "Warning: "),
            LogLevel::Error => (self.colors.red, "Error: "),
            LogLevel::Dim => (self.colors.dim, ""),
            LogLevel::Success => (self.colors.green, ""),
            LogLevel::Bold => (self.colors.reset, ""),
            LogLevel::Custom(code) => (code, ""),
            LogLevel::CriticalError => (self.colors.red, "Error: "),
        };

        let final_color_start = match level {
            LogLevel::Success => format!("{}{}", base_color, self.colors.bold),
            LogLevel::Bold => self.colors.bold.to_string(),
            _ => base_color.to_string(),
        };

        let output_str = format!(
            "{}{}{}{}{}",
            final_color_start, indent, text_prefix, message, self.colors.reset
        );

        if matches!(level, LogLevel::Error | LogLevel::CriticalError) {
            eprintln!("{}", output_str);
        } else {
            println!("{}", output_str);
        }
    }

    pub fn info(&self, message: &str) {
        self.print_message(LogLevel::Info, 1, message);
    }
    pub fn warn(&self, message: &str) {
        self.print_message(LogLevel::Warn, 1, message);
    }
    pub fn error(&self, message: &str) {
        self.print_message(LogLevel::Error, 1, message);
    }
    pub fn dim(&self, message: &str) {
        self.print_message(LogLevel::Dim, 1, message);
    }
    pub fn success(&self, message: &str) {
        self.print_message(LogLevel::Success, 1, message);
    }
    pub fn bold(&self, message: &str) {
        self.print_message(LogLevel::Bold, 1, message);
    }
    pub fn custom(&self, color_code: &'static str, message: &str) {
        self.print_message(LogLevel::Custom(color_code), 1, message);
    }
    pub fn critical_error(&self, message: &str) {
        self.print_message(LogLevel::CriticalError, 0, message);
    }
    pub fn dim_level2(&self, message: &str) {
        self.print_message(LogLevel::Dim, 2, message);
    }
    pub fn info_level2(&self, message: &str) {
        self.print_message(LogLevel::Info, 2, message);
    }

    pub fn raw(&self, message: &str) {
        if !QUIET_MODE.load(Ordering::Relaxed) {
            println!("{}", message);
        }
    }
}

pub static LOGGER_INSTANCE: Logger = Logger::new();
