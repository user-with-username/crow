use blake3::Hasher;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn hash_source(
    src: &Path,
    headers: &[PathBuf],
    compiler: &str,
    flags: &[String],
) -> anyhowed::Result<String> {
    let mut hasher = Hasher::new();

    hasher.update(compiler.as_bytes());

    for f in flags {
        hasher.update(f.as_bytes());
    }

    hasher.update(&fs::read(src)?);

    let mut headers = headers.to_vec();
    headers.sort();

    for h in headers {
        match fs::read(&h) {
            Ok(data) => {
                hasher.update(&data);
            }
            Err(_) => {
                hasher.update(b"<missing-header>");
                hasher.update(h.to_string_lossy().as_bytes());
            }
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

pub fn hash_files(paths: &[PathBuf]) -> anyhowed::Result<String> {
    let mut sorted_paths = paths.to_vec();
    sorted_paths.sort();

    let mut hasher = Hasher::new();
    for path in sorted_paths {
        hasher.update(path.to_string_lossy().as_bytes());
        match fs::read(&path) {
            Ok(data) => {
                hasher.update(&data);
            }
            Err(_) => {
                hasher.update(b"<missing-file>");
                hasher.update(path.to_string_lossy().as_bytes());
            }
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}
