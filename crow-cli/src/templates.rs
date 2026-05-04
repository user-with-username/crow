use anyhow::Result;
use std::fs;
use std::path::Path;

pub const MAIN_CPP: &str = r#"#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
"#;

pub const CROW_TOML: &str = r#"[package]
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

pub fn create_src_directory(src_dir: &Path) -> Result<()> {
    if !src_dir.exists() {
        fs::create_dir_all(src_dir)?;
    }
    Ok(())
}

pub fn write_main_cpp(src_dir: &Path) -> Result<()> {
    let main_cpp = src_dir.join("main.cpp");
    fs::write(main_cpp, MAIN_CPP)?;
    Ok(())
}

pub fn write_crow_toml(config_path: &Path, package_name: &str, overwrite: bool) -> Result<()> {
    if !overwrite && config_path.exists() {
        return Ok(());
    }

    let content = CROW_TOML.replace("{name}", package_name);
    fs::write(config_path, content)?;
    Ok(())
}

pub fn write_gitignore(base_path: &Path, overwrite: bool) -> Result<()> {
    let gitignore_path = base_path.join(".gitignore");

    if !overwrite && gitignore_path.exists() {
        return Ok(());
    }

    fs::write(gitignore_path, GITIGNORE)?;
    Ok(())
}

pub fn create_project_structure(
    base_path: &Path,
    package_name: &str,
    force_overwrite: bool,
) -> Result<()> {
    let src_dir = base_path.join("src");
    create_src_directory(&src_dir)?;
    write_main_cpp(&src_dir)?;

    let config_path = base_path.join("crow.toml");
    write_crow_toml(&config_path, package_name, force_overwrite)?;

    write_gitignore(base_path, force_overwrite)?;

    Ok(())
}
