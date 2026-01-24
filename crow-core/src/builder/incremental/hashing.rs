use blake3::Hasher;
use std::{fs, path::{Path, PathBuf}};

pub fn hash_source(
    src: &Path,
    headers: &[PathBuf],
    compiler: &str,
    flags: &[String],
) -> anyhow::Result<String> {
    let mut hasher = Hasher::new();

    // compiler identity
    hasher.update(compiler.as_bytes());

    // flags
    for f in flags {
        hasher.update(f.as_bytes());
    }

    // source
    hasher.update(&fs::read(src)?);

    // headers (sorted for stability)
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