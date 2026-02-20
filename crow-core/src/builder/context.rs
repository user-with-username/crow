use crate::builder::{
    flags::CompilerFlags,
    incremental::{hash_source, IncrementalManager},
    paths::{ObjectFilePath, SourceFilePath},
    tasks::{IncludeTask, SourceCompilationTask},
};
use crate::project::Project;
use anyhow::{Context, Result};
use crow_utils::{show_output, ProgressBar};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct CompilationContext<'a> {
    compiler_exe: &'a str,
    project: &'a Project,
    deps_dir: PathBuf,
    base_flags: Vec<String>,
    is_msvc: bool,
}

impl<'a> CompilationContext<'a> {
    pub(crate) fn new(compiler_exe: &'a str, project: &'a Project) -> Result<Self> {
        let profile_dir = project.profile_dir();
        let deps_dir = profile_dir.join("deps");
        let is_msvc = project.compiler_kind().is_msvc();

        fs::create_dir_all(&deps_dir)?;

        let mut flags = CompilerFlags::new(project.compiler_kind());
        flags.standard_flags();
        project.profile.apply_to_compile_flags(&mut flags);

        for inc in &project.config.build.include_dirs {
            flags.include_path(inc.to_string_lossy().into_owned());
        }

        if project.config.build.warnings_as_errors {
            flags.warnings_as_errors();
        }

        for def in &project.config.build.preprocessor_defines {
            let (name, value) = def
                .split_once('=')
                .map(|(k, v)| (k, Some(v)))
                .unwrap_or((def.as_str(), None));
            flags.define(name, value);
        }

        for f in project.config.build.compiler.flags() {
            flags.add_raw(f.clone());
        }

        Ok(Self {
            compiler_exe,
            project,
            deps_dir,
            base_flags: flags.build(),
            is_msvc,
        })
    }

    pub(crate) fn compile_unit(
        &self,
        source_path: &Path,
        cache_manager: &IncrementalManager,
        progress: &ProgressBar,
    ) -> Result<ObjectFilePath> {
        let source = SourceFilePath(source_path.to_path_buf());
        let mut task = SourceCompilationTask::prepare(
            &source,
            self.compiler_exe,
            &self.project.config.build,
            &self.deps_dir,
            self.project,
            &self.project.profile,
        )?;

        let object_path = task.object.clone();
        let pre_hash = self.compute_hash_before_compile(&task)?;

        let needs_compile =
            cache_manager.should_compile(&source, &Some(pre_hash.clone()), &object_path);

        if needs_compile {
            self.execute_compilation(&mut task, progress)?;

            let final_hash = if self.is_msvc {
                pre_hash
            } else {
                let headers = IncludeTask::new(task.dep_file.clone().into())
                    .collect_headers()
                    .unwrap_or_default();
                hash_source(
                    task.source.as_path(),
                    &headers,
                    self.compiler_exe,
                    &self.base_flags,
                )?
            };

            cache_manager.record_success(&source, task.to_cache_entry(final_hash));
        }

        progress.inc();
        Ok(object_path)
    }

    fn compute_hash_before_compile(&self, task: &SourceCompilationTask) -> Result<String> {
        let headers = if !self.is_msvc && task.dep_file.exists() {
            IncludeTask::new(task.dep_file.clone().into())
                .collect_headers()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        hash_source(
            task.source.as_path(),
            &headers,
            self.compiler_exe,
            &self.base_flags,
        )
    }

    fn execute_compilation(
        &self,
        task: &mut SourceCompilationTask,
        progress: &ProgressBar,
    ) -> Result<()> {
        let output = task
            .command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .with_context(|| {
                format!(
                    "Failed to run compiler for {}",
                    task.source.as_path().display()
                )
            })?;

        if !output.status.success() {
            progress.finish();
            show_output!(output, self.project);
            anyhow::bail!(
                "Compilation failed for {} (exit code: {:?})",
                task.source.as_path().display(),
                output.status.code()
            );
        }
        Ok(())
    }
}