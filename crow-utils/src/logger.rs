use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LogLevel {
    Error,
    Info,
    Dim,
    Warn,
    Success,
    Bold,
    Custom(&'static str),
}

#[derive(Clone)]
pub struct Logger {
    pub quiet: bool,
    pub verbose: bool,
}

pub trait IntoIndent {
    fn into_indent(self) -> Option<u8>;
}

impl IntoIndent for () {
    fn into_indent(self) -> Option<u8> { None }
}

impl IntoIndent for u8 {
    fn into_indent(self) -> Option<u8> { Some(self) }
}

pub trait IntoLogLevel {
    fn into_level(self) -> Option<LogLevel>;
}

impl IntoLogLevel for () {
    fn into_level(self) -> Option<LogLevel> { None }
}

impl IntoLogLevel for LogLevel {
    fn into_level(self) -> Option<LogLevel> { Some(self) }
}

impl Logger {
    pub fn new() -> Self {
        Self {
            quiet: false,
            verbose: false,
        }
    }

    pub fn quiet(&mut self, quiet: bool) -> &mut Self {
        self.quiet = quiet;
        self
    }

    pub fn verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    pub fn log<L, I>(&self, level: L, msg: impl fmt::Display, indent: I)
    where
        L: IntoLogLevel,
        I: IntoIndent,
    {
        if self.quiet {
            return;
        }

        let spaces = indent.into_indent()
            .map_or(String::new(), |n| " ".repeat(n as usize * 2));

        let formatted = match level.into_level() {
            None => format!("{}{}", spaces, msg),
            Some(LogLevel::Error) => format!("\x1b[31mError: {}{}\x1b[0m", spaces, msg),
            Some(LogLevel::Info) => format!("\x1b[36m{}{}\x1b[0m", spaces, msg),
            Some(LogLevel::Dim) => format!("\x1b[2m{}{}\x1b[0m", spaces, msg),
            Some(LogLevel::Warn) => format!("\x1b[33m{}Warning: {}\x1b[0m", spaces, msg),
            Some(LogLevel::Success) => format!("\x1b[32m\x1b[1m{}{}\x1b[0m", spaces, msg),
            Some(LogLevel::Bold) => format!("\x1b[1m{}{}\x1b[0m", spaces, msg),
            Some(LogLevel::Custom(ansi_code)) => format!("{}{}{}\x1b[0m", ansi_code, spaces, msg),
        };

        println!("{}", formatted);
    }
}