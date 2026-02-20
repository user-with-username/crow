use crate::{
    builder::paths::{ObjectFilePath, SourceFilePath},
    builder::{
        incremental::{hash_source, IncrementalManager},
        tasks::{IncludeTask, SourceCompilationTask},
    },
    project::Project,
};
use anyhow::{Context, Result};
use crow_utils::{show_output, ProgressBar};
use std::process::Stdio;

pub struct CompilationBuilder<'a> {
    compiler_exe: &'a str,
    project: &'a Project,
}

impl<'a> CompilationBuilder<'a> {
    pub fn new(compiler_exe: &'a str, project: &'a Project) -> Self {
        Self {
            compiler_exe,
            project,
        }
    }

    pub fn compile(&self) -> Result<Vec<ObjectFilePath>> {
        self.project.create_dirs()?;
        let profile_dir = self.project.profile_dir();
        let deps_dir = profile_dir.join("deps");
        if !self.project.compiler_kind().is_msvc() {
            std::fs::create_dir_all(&deps_dir)?;
        }

        let cache_path = profile_dir.join(".fingerprint.json");
        let mut cache_manager: IncrementalManager =
            IncrementalManager::new(&cache_path, self.project.profile.incremental());

        let sources: Vec<_> = self.project.find_sources().into_iter().collect();
        let total = sources.len();

        let progress =
            ProgressBar::new(total, format!("{}(bin)", self.project.config.package.name));

        let mut objects = Vec::new();
        let is_msvc = self.project.compiler_kind().is_msvc();

        for source_path in sources {
            let source = SourceFilePath(source_path);
            let task = SourceCompilationTask::prepare(
                &source,
                self.compiler_exe,
                &self.project.config.build,
                &deps_dir,
                self.project,
                &self.project.profile,
            )?;

            let object_path = task.object.clone();

            let headers = if !is_msvc && task.dep_file.exists() {
                IncludeTask::new(task.dep_file.clone().into())
                    .collect_headers()
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let flags = self.project.config.build.compiler.flags();
            let current_hash =
                hash_source(task.source.as_path(), &headers, self.compiler_exe, &flags)?;

            let needs_compile =
                cache_manager.should_compile(&source, &Some(current_hash.clone()), &object_path);

            if needs_compile {
                let mut task = task;
                if let Err(e) = self.execute_compilation(&mut task, &progress) {
                    progress.finish();
                    return Err(e);
                }

                let final_hash = if is_msvc {
                    current_hash
                } else {
                    let include_task = IncludeTask::new(task.dep_file.clone().into());
                    let headers = include_task.collect_headers()?;
                    hash_source(task.source.as_path(), &headers, self.compiler_exe, &flags)?
                };

                cache_manager.record_success(&source, task.to_cache_entry(final_hash));
            }

            objects.push(object_path);
            progress.inc();
        }

        progress.finish();
        cache_manager.finalize()?;

        Ok(objects)
    }

    fn execute_compilation(
        &self,
        task: &mut SourceCompilationTask,
        progress: &ProgressBar,
    ) -> Result<()> {
        let output = task
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| {
                format!(
                    "Cannot run compiler for {}",
                    task.source.as_path().display()
                )
            })?;

        if !output.status.success() {
            progress.finish();

            show_output!(output, self.project);

            let exit_code = output.status.code().unwrap_or(-1);
            anyhow::bail!(
                "Compilation failed for {} (exit code: {})",
                task.source.as_path().display(),
                exit_code
            );
        }

        Ok(())
    }
}
