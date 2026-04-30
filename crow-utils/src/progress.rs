use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct ProgressBar {
    total: usize,
    current: AtomicUsize,
    last_drawn: AtomicUsize,
    label: Mutex<String>,
    io_lock: Mutex<()>,
}

impl ProgressBar {
    pub fn new(total: usize, label: impl Into<String>) -> Self {
        Self {
            total,
            current: AtomicUsize::new(0),
            last_drawn: AtomicUsize::new(0),
            label: Mutex::new(label.into()),
            io_lock: Mutex::new(()),
        }
    }

    pub fn inc(&self) {
        let val = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        self.draw(val);
    }

    pub fn inc_with_label(&self, label: impl Into<String>) {
        self.set_label(label);
        self.inc();
    }

    pub fn set_label(&self, label: impl Into<String>) {
        let mut current = self.label.lock().unwrap();
        *current = label.into();
    }

    pub fn finish(&self) {
        let _lock = self.io_lock.lock().unwrap();
        print!("\r\x1b[2K\n");
        let _ = io::stdout().flush();
    }

    fn draw(&self, current: usize) {
        let _lock = self.io_lock.lock().unwrap();
        let label = self.label.lock().unwrap().clone();

        let last = self.last_drawn.load(Ordering::SeqCst);
        if current < last && current < self.total {
            return;
        }
        self.last_drawn.store(current, Ordering::SeqCst);

        let width = 30;
        let total_safe = self.total.max(1);
        let filled = (current * width) / total_safe;
        let filled = filled.min(width);

        let mut bar_content = "=".repeat(filled);
        if filled < width {
            bar_content.push('>');
            bar_content.push_str(&" ".repeat(width - filled - 1));
        } else {
            if bar_content.len() < width {
                bar_content.push('=');
            }
        }

        print!(
            "\r\x1b[2K\x1b[1;96m{:>12}\x1b[0m [{}] {}/{}: {}{}",
            "Building", bar_content, current, self.total, label,
            " ".repeat(10)
        );
        let _ = io::stdout().flush();
    }
}