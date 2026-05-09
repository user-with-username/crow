use std::borrow::Cow;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn escape(arg: &str) -> Cow<'_, str> {
    if arg.is_empty() {
        return Cow::Borrowed("\"\"");
    }
    if !arg.contains([' ', '\t', '"']) {
        return Cow::Borrowed(arg);
    }
    Cow::Owned(format!("\"{}\"", arg.replace('"', "\"\"")))
}

pub fn write_to<P: AsRef<Path>, T: AsRef<str>>(
    profile_dir: P,
    filename: &str,
    args: &[T],
    is_msvc: bool,
) -> std::io::Result<PathBuf> {
    let path = profile_dir.as_ref().join("args").join(filename);
    fs::create_dir_all(path.parent().unwrap())?;

    let mut writer = BufWriter::new(File::create(&path)?);
    for arg in args {
        let s = arg.as_ref();
        let normalized = if !is_msvc && s.contains('\\') {
            Cow::Owned(s.replace('\\', "/"))
        } else {
            Cow::Borrowed(s)
        };
        writeln!(writer, "{}", escape(&normalized))?;
    }
    writer.flush()?;

    Ok(path)
}
