use anyhow::{Context, Result};
use crow_utils::DiagnosticHighlighter;
use std::process::Stdio;

use crate::{
    builder::{
        incremental::{IncrementalManager, hash_source},
        tasks::{SourceCompilationTask, IncludeTask},
    },
    builder::paths::{ObjectFilePath, SourceFilePath},
    project::Project,
};

pub struct CompilationBuilder<'a> {
    compiler_exe: &'a str,
    project: &'a Project,
}

impl<'a> CompilationBuilder<'a> {
    pub fn new(compiler_exe: &'a str, project: &'a Project) -> Self {
        Self { compiler_exe, project }
    }

    pub fn compile(&self) -> Result<Vec<ObjectFilePath>> {
        self.project.create_dirs()?;

        let profile_dir = self.project.profile_dir();
        let deps_dir = profile_dir.join("deps");
        std::fs::create_dir_all(&deps_dir)?;

        let cache_path = profile_dir.join(".fingerprint.json");
        let mut cache_manager = IncrementalManager::new(&cache_path, self.project.release);

        let mut objects = Vec::new();

        for source_path in self.project.find_sources() {
            let source = SourceFilePath(source_path);

            let task = SourceCompilationTask::prepare(
                &source,
                self.compiler_exe,
                &self.project.config.build,
                &deps_dir,
                self.project.release,
            )?;

            let object_path = task.object.clone();

            let headers = if task.dep_file.exists() {
                IncludeTask::new(task.dep_file.clone()).collect_headers().unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let flags = self.project.config.build.flags.clone();
            let current_hash = hash_source(
                task.source.as_path(),
                &headers,
                self.compiler_exe,
                &flags,
            )?;

            let needs_compile = cache_manager.should_compile(&source, &Some(current_hash), &object_path);

            if needs_compile {
                let mut task = task;
                self.execute_compilation(&mut task)?;
                
                let include_task = IncludeTask::new(task.dep_file.clone());
                let headers = include_task.collect_headers()?;
                let final_hash = hash_source(
                    task.source.as_path(),
                    &headers,
                    self.compiler_exe,
                    &flags,
                )?;
                
                cache_manager.record_success(&source, task.to_cache_entry(final_hash));
            }

            objects.push(object_path);
        }

        cache_manager.finalize()?;
        Ok(objects)
    }

    fn execute_compilation(&self, task: &mut SourceCompilationTask) -> Result<()> {
        let output = task
            .command
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Cannot run compiler for {}", task.source.as_path().display()))?;

        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines() {
                eprintln!("{}", DiagnosticHighlighter::colorize(line));
            }
        }

        if !output.status.success() {
            anyhow::bail!("Compilation failed: {}", task.source.as_path().display());
        }

        Ok(())
    }
}
