use crate::{flags::Flags, project::Project};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct CompilationBuilder<'a> {
    compiler_exe: &'a str,
    project: &'a Project,
    verbose: u8,
}

impl<'a> CompilationBuilder<'a> {
    pub fn new(
        compiler_exe: &'a str,
        project: &'a Project,
        verbose: u8,
    ) -> Self {
        Self { compiler_exe, project, verbose }
    }

    pub fn compile(&self) -> Result<Vec<PathBuf>> {
        self.project.create_dirs()?;

        let mut objects = Vec::new();

        for src in self.project.find_sources() {
            let obj = self.project
                .build_dir()
                .join(src.file_stem().unwrap())
                .with_extension("o");

            let mut flags = Flags::new();
            
            flags.compile_only()
                .object_output(obj.to_string_lossy().to_string());
            
            let build = &self.project.config.build;
            
            for dir in &build.include_dirs {
                flags.include_path(dir.to_string_lossy().to_string());
            }
            
            for raw_flag in &build.flags {
                flags.add_raw(raw_flag);
            }
            
            for lib in &build.libs {
                flags.add_raw(&format!("-l{}", lib));
            }
            
            for dir in &build.lib_dirs {
                flags.add_raw(&format!("-L{}", dir.to_string_lossy()));
            }
            
            let args = flags.build();
            
            let mut cmd = Command::new(self.compiler_exe);
            cmd.args(&args);
            cmd.arg(src.to_string_lossy().to_string());

            if self.verbose > 1 {
                crow_utils::status!("Command", "{:?}", cmd);
            }

            let status = cmd.status().context("compiler failed")?;
            if !status.success() {
                anyhow::bail!("compilation failed: {}", src.display());
            }

            objects.push(obj);
        }

        Ok(objects)
    }
}