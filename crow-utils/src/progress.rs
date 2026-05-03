use indicatif::{MultiProgress, ProgressBar as NativeProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct ProgressBar {
    pb: NativeProgressBar,
    _mp: Arc<MultiProgress>,
}

impl ProgressBar {
    pub fn new(total: usize, label: impl Into<String>) -> Self {
        let mp = Arc::new(MultiProgress::new());
        let pb = mp.add(NativeProgressBar::new(total as u64));

        let style = ProgressStyle::with_template(
            "\x1b[1;96m{prefix:>12}\x1b[0m [{bar:30}] {pos}/{len}: {msg}",
        )
        .unwrap()
        .progress_chars("=> ");

        pb.set_style(style);
        pb.set_prefix("Building");
        pb.set_message(label.into());

        Self { pb, _mp: mp }
    }

    pub fn inc(&self) {
        self.pb.inc(1);
    }

    pub fn inc_with_label(&self, label: impl Into<String>) {
        self.pb.set_message(label.into());
        self.pb.inc(1);
    }

    pub fn set_label(&self, label: impl Into<String>) {
        self.pb.set_message(label.into());
    }

    pub fn finish(&self) {
        self.pb.finish_and_clear();
    }

    pub fn status(&self, status: &str, msg: &str) {
        self.pb.suspend(|| {
            println!("\x1b[1;92m{:>12}\x1b[0m {}", status, msg);
        });
    }
}
