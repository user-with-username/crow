use crate::{compiler_kind::CompilerKind, flags::Flags, project::Project};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct CompilationBuilder<'a> {
    compiler_kind: CompilerKind,
    compiler_exe: &'a str,
    project: &'a Project,
    verbose: u8,
}

impl<'a> CompilationBuilder<'a> {
    pub fn new(
        compiler_kind: CompilerKind,
        compiler_exe: &'a str,
        project: &'a Project,
        verbose: u8,
    ) -> Self {
        Self { compiler_kind, compiler_exe, project, verbose }
    }

    pub fn compile(&self) -> Result<Vec<PathBuf>> {
    self.project.create_dirs()?;

    let profile = self.project.get_profile();
    let flags = Flags::new(profile, &self.project.config.build, self.compiler_kind);

    let mut objects = Vec::new();

    for src in self.project.find_sources() {
        let obj = self.project
            .build_dir()
            .join(src.file_stem().unwrap())
            .with_extension("o");

        let mut cmd = Command::new(self.compiler_exe);
        cmd.args(flags.compile_flags())
            .args(flags.include_dir_flags())
            .arg(&src)
            .arg("-o")
            .arg(&obj);

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
