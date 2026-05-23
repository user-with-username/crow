use anyhowed::Result;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

pub const MAIN_CPP: &str = r#"#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
"#;

pub const CROW_TOML_TEMPLATE: &str = r#"[package]
name = "{name}"
version = "0.1.0"
standard = "20"

[dependencies]
"#;

pub const GITIGNORE: &str = r#"target/
*.exe
*.o
*.obj
*.a
*.so
*.dylib
*.dll
.DS_Store
"#;

struct FileSpec<'a> {
    path: PathBuf,
    content: Cow<'a, str>,
}

impl<'a> FileSpec<'a> {
    fn new(path: PathBuf, content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            path,
            content: content.into(),
        }
    }

    fn write(&self, overwrite: bool) -> Result<()> {
        if !overwrite && self.path.exists() {
            return Ok(());
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, self.content.as_bytes())?;
        Ok(())
    }
}

pub fn create_project_structure(
    base_path: &Path,
    package_name: &str,
    overwrite: bool,
) -> Result<()> {
    [
        FileSpec::new(base_path.join("src/main.cpp"), MAIN_CPP),
        FileSpec::new(
            base_path.join("crow.toml"),
            Cow::Owned(CROW_TOML_TEMPLATE.replace("{name}", package_name)),
        ),
        FileSpec::new(base_path.join(".gitignore"), GITIGNORE),
    ]
    .into_iter()
    .try_for_each(|spec| spec.write(overwrite))
}
