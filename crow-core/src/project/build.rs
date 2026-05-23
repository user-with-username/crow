use crate::builder::{CompilationBuilder, LinkingBuilder};
use crate::project::Project;
use anyhowed::Result;
use crow_utils::progress::ProgressBar;
use std::path::PathBuf;

impl Project {
    pub fn compiler_kind(&self) -> crate::builder::kinds::compiler_kind::CompilerKind {
        self.toolchain()
            .map(|t| t.compiler_kind())
            .unwrap_or_else(|_| self.config.build.compiler.kind())
    }

    pub fn linker_kind(&self) -> crate::builder::kinds::linker_kind::LinkerKind {
        self.toolchain()
            .map(|t| t.linker_kind())
            .unwrap_or_else(|_| self.config.build.linker.kind())
    }

    pub fn compiler_path(&self) -> &str {
        self.toolchain()
            .map(|t| t.compiler_path())
            .expect("toolchain detection failed")
    }

    pub fn linker_path(&self) -> &str {
        if let Some(path) = self.config.build.linker.path() {
            return path;
        }
        let toolchain = self
            .toolchain()
            .expect("toolchain detection failed");
        if self.compiler_kind().is_msvc() {
            toolchain.linker_path()
        } else {
            toolchain.compiler_path()
        }
    }

    pub fn archiver_path(&self) -> &str {
        self.toolchain()
            .map(|t| t.archiver_path())
            .expect("toolchain detection failed")
    }

    pub fn archiver_kind(&self) -> crate::builder::kinds::archiver_kind::ArchiverKind {
        self.toolchain()
            .map(|t| t.archiver_kind())
            .unwrap_or_else(|_| self.config.build.archiver.kind())
    }

    pub fn system_include_dirs(&self) -> Vec<PathBuf> {
        self.toolchain()
            .map(|t| t.system_include_dirs())
            .unwrap_or_default()
    }

    pub fn configure_parallelism(&self, jobs: Option<usize>) {
        if self.config.build.parallelism {
            if let Some(jobs) = jobs {
                let _ = rayon::ThreadPoolBuilder::new()
                    .num_threads(jobs)
                    .build_global();
            }
        }
    }

    pub fn compile_and_link(&self, lock_hash: &str, progress: Option<&ProgressBar>) -> Result<()> {
        let compiler_exe = self
            .config
            .build
            .compiler
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.compiler_path());

        let archiver_exe = self
            .config
            .build
            .archiver
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.archiver_path());

        let linker_exe = self
            .config
            .build
            .linker
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.linker_path());

        self.create_dirs()?;

        let objects = CompilationBuilder::new(compiler_exe, self).compile(progress)?;

        LinkingBuilder::new(linker_exe, archiver_exe, self, &objects, progress).link()?;

        self.save_project_state(lock_hash)?;

        Ok(())
    }
}
