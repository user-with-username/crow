use anyhow::Result;
use std::path::Path;

pub struct CompilationDatabase {
    entries: Vec<ClangdEntry>,
}

#[derive(serde::Serialize)]
struct ClangdEntry {
    directory: String,
    arguments: Vec<String>,
    file: String,
}

impl CompilationDatabase {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn clean_path<P: AsRef<Path>>(path: P) -> String {
        let path = path.as_ref();
        let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let s = abs.to_string_lossy().to_string();
        s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
    }

    pub fn add_entry(&mut self, project_root: &Path, file_path: &Path, mut args: Vec<String>) {
        let root_str = Self::clean_path(project_root);
        let abs_file = Self::clean_path(project_root.join(file_path));

        if !args.is_empty() {
            args.insert(1, "--driver-mode=cl".to_string());
        } // looks like cringe, but it makes clangd work better

        self.entries.push(ClangdEntry {
            directory: root_str,
            arguments: args,
            file: abs_file,
        });
    }

    pub fn save(self, project_root: &Path) -> Result<()> {
        let json_path = project_root.join("compile_commands.json");
        let json_data = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(json_path, json_data)?;
        Ok(())
    }
}
