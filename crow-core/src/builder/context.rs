use crate::builder::{
    flags::CompilerFlags,
    incremental::{hash_source, IncrementalManager},
    paths::{ObjectFilePath, SourceFilePath},
    tasks::{database::CompilationDatabase, source::SourceCompilationTask, IncludeTask},
};
use crate::project::Project;
use anyhow::{anyhow, Result};
use crow_utils::progress::ProgressBar;
use crow_utils::show_output;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

fn apply_language_standard(flags: &mut CompilerFlags, source_path: &Path, standard: Option<&str>) {
    let Some(standard) = standard else { return };

    let extension = source_path.extension().and_then(|ext| ext.to_str());

    match extension {
        Some(ext) if ["cpp", "cc", "cxx", "c++"].contains(&ext) => {
            flags.cxx_standard(standard);
        }
        Some("c") => {
            flags.c_standard(standard);
        }
        _ => {}
    }
}

pub(crate) struct CompilationContext<'a> {
    pub(crate) compiler_exe: &'a str,
    pub(crate) project: &'a Project,
    pub(crate) deps_dir: PathBuf,
    pub(crate) base_flags: Vec<String>,
    pub(crate) is_msvc: bool,
    pub(crate) progress: Option<&'a ProgressBar>,
}

impl<'a> CompilationContext<'a> {
    pub(crate) fn new(
        compiler_exe: &'a str,
        project: &'a Project,
        progress: Option<&'a ProgressBar>,
    ) -> Result<Self> {
        let profile_dir = project.profile_dir();
        let deps_dir = profile_dir.join("deps");
        let is_msvc = project.compiler_kind().is_msvc();

        fs::create_dir_all(&deps_dir)?;

        let mut flags = CompilerFlags::new(project.compiler_kind());
        flags.standard_flags();
        project.profile.apply_to_compile_flags(&mut flags);

        if project.package.r#type.is_shared() && !project.compiler_kind().is_msvc() {
            flags.position_independent_code();
        }

        for inc in &project.config.build.include_dirs {
            let abs_inc = if inc.is_relative() {
                project.root.join(inc)
            } else {
                inc.clone()
            };
            flags.include_path(CompilationDatabase::clean_path(abs_inc));
        }

        if project.config.build.warnings_as_errors {
            flags.warnings_as_errors();
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
            progress,
        })
    }

    pub(crate) fn compile_unit(
        &self,
        source_path: &Path,
        cache_manager: &IncrementalManager,
    ) -> Result<(ObjectFilePath, PathBuf, Vec<String>)> {
        let source = SourceFilePath(source_path.to_path_buf());

        let mut lang_flags = CompilerFlags::new(self.project.compiler_kind());
        apply_language_standard(
            &mut lang_flags,
            source_path,
            self.project.package.standard(),
        );
        let extra_flags = lang_flags.build();

        let mut task = SourceCompilationTask::prepare(
            &source,
            self.compiler_exe,
            &self.project.config.build,
            &self.deps_dir,
            self.project,
            &self.project.profile,
            &extra_flags,
        )?;

        let object_path = task.object.clone();
        let mut args = vec![self.compiler_exe.to_string()];
        args.extend(task.args.clone());

        let pre_hash = self._compute_hash_before_compile(&task)?;

        if cache_manager.should_compile(&source, &Some(pre_hash), &object_path) {
            let stdout_lines = self.execute_compilation(&mut task)?;

            if self.is_msvc {
                IncludeTask::save_deps(task.dep_file.as_path(), &stdout_lines)?;
            }

            let final_hash = self._compute_hash_before_compile(&task)?;
            cache_manager.record_success(&source, task.to_cache_entry(final_hash));
        }

        Ok((object_path, source_path.to_path_buf(), args))
    }

    fn execute_compilation(&self, task: &mut SourceCompilationTask) -> Result<Vec<String>> {
        let output = task
            .command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()?;

        if !output.status.success() {
            if let Some(pb) = self.progress {
                pb.finish();
            }
            show_output!(output, self.project);
            return Err(anyhow!(
                "Compilation failed for {}",
                task.source.as_path().display()
            ));
        }

        let stdout_reader = BufReader::new(&output.stdout[..]);
        Ok(stdout_reader.lines().filter_map(|l| l.ok()).collect())
    }

    fn _compute_hash_before_compile(&self, task: &SourceCompilationTask) -> Result<String> {
        let mut headers = Vec::new();
        if task.dep_file.exists() {
            if self.is_msvc {
                if let Ok(content) = fs::read_to_string(task.dep_file.as_path()) {
                    headers = content.lines().map(PathBuf::from).collect();
                }
            } else {
                headers = IncludeTask::new(task.dep_file.clone().into())
                    .collect_headers()
                    .unwrap_or_default();
            }
        }
        hash_source(
            task.source.as_path(),
            &headers,
            self.compiler_exe,
            &self.base_flags,
        )
    }
}