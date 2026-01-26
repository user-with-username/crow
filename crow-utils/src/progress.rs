use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ProgressBar {
    total: usize,
    current: Arc<AtomicUsize>,
    label: String,
}

impl ProgressBar {
    pub fn new(total: usize, label: impl Into<String>) -> Self {
        Self {
            total,
            current: Arc::new(AtomicUsize::new(0)),
            label: label.into(),
        }
    }

    pub fn inc(&self) {
        let current = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        self.draw(current);
    }

    pub fn finish(&self) {
        self.draw(self.total);
        print!("\r{}\r", " ".repeat(100));
        io::stdout().flush().unwrap();
    }

    fn draw(&self, current: usize) {
        let width = 30;
        let filled = (current * width) / self.total.max(1);

        let bar_content = format!(
            "{}{}{}",
            "=".repeat(filled),
            ">",
            " ".repeat(width.saturating_sub(filled))
        );

        let bar = format!(
            "\r\x1b[1;96m{:>12}\x1b[0m [{}] {}/{}: {}",
            "Building", bar_content, current, self.total, self.label
        );

        print!("{}", bar);
        io::stdout().flush().unwrap();
    }
}
