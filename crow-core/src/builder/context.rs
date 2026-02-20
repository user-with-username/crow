use crate::builder::{
    flags::CompilerFlags,
    incremental::{hash_source, IncrementalManager},
    paths::{ObjectFilePath, SourceFilePath},
    tasks::source::{CompileCommand, SourceCompilationTask},
    tasks::IncludeTask,
};
use crate::project::Project;
use anyhow::Result;
use crow_utils::{normalize_path, show_output, ProgressBar};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub(crate) struct CompilationContext<'a> {
    pub(crate) compiler_exe: &'a str,
    pub(crate) project: &'a Project,
    pub(crate) deps_dir: PathBuf,
    pub(crate) base_flags: Vec<String>,
    pub(crate) is_msvc: bool,
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
            let (name, value) = def.split_once('=').map(|(k, v)| (k, Some(v))).unwrap_or((def.as_str(), None));
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
    ) -> Result<(ObjectFilePath, CompileCommand)> {
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
        let compile_command = task.to_compile_command(&self.project.root);
        
        let pre_hash = self.compute_hash_before_compile(&task)?;
        let needs_compile = cache_manager.should_compile(&source, &Some(pre_hash.clone()), &object_path);

        if needs_compile {
            let stdout_lines = self.execute_compilation(&mut task, progress)?;

            if self.is_msvc {
                self.save_msvc_deps(&task.dep_file.as_path(), &stdout_lines)?;
            }

            let final_hash = self.compute_hash_before_compile(&task)?;
            cache_manager.record_success(&source, task.to_cache_entry(final_hash));
        }

        progress.inc();
        Ok((object_path, compile_command))
    }

    fn compute_hash_before_compile(&self, task: &SourceCompilationTask) -> Result<String> {
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

    fn execute_compilation(&self, task: &mut SourceCompilationTask, progress: &ProgressBar) -> Result<Vec<String>> {
        let output = task.command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()?;

        let stdout_reader = BufReader::new(&output.stdout[..]);
        let stdout_lines: Vec<String> = stdout_reader.lines().filter_map(|l| l.ok()).collect();

        if !output.status.success() {
            progress.finish();
            show_output!(output, self.project);
            anyhow::bail!("Compilation failed for {}", task.source.as_path().display());
        }
        Ok(stdout_lines)
    }

    fn save_msvc_deps(&self, dep_path: &Path, lines: &[String]) -> Result<()> {
        let mut deps = Vec::new();
        for line in lines {
            if let Some(path_str) = line.strip_prefix("Note: including file: ") {
                let path = path_str.trim();
                deps.push(normalize_path(path));
            }
        }
        if !deps.is_empty() {
            let mut f = fs::File::create(dep_path)?;
            for d in deps {
                writeln!(f, "{}", d)?;
            }
        }
        Ok(())
    }
}