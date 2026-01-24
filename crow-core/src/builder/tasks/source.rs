use crate::builder::{
    paths::ObjectFileNaming,
    incremental::CacheEntry,
    paths::{ObjectFilePath, SourceFilePath},
    flags::Flags,
};
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
pub struct SourceCompilationTask {
    pub source: SourceFilePath,
    pub object: ObjectFilePath,
    pub dep_file: PathBuf,
    pub command: Command,
}

impl SourceCompilationTask {
    pub fn prepare(
        source: &SourceFilePath,
        compiler_exe: &str,
        build_config: &crate::config::BuildConfig,
        deps_dir: &Path,
        release: bool,
    ) -> Result<Self> {
        let mut flags = Flags::new();
        flags.compile_only();

        for inc in &build_config.include_dirs {
            flags.include_path(inc.to_string_lossy().into_owned());
        }

        for raw_flag in &build_config.flags {
            flags.add_raw(raw_flag.clone());
        }

        let dep_file = deps_dir.join(
            source
                .as_path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
                + ".d",
        );

        flags.dependency_info(dep_file.to_string_lossy().into_owned());

        let object_path =
            ObjectFileNaming::generate(deps_dir, source, None, release);

        flags.object_output(object_path.to_string_lossy().into_owned());

        let mut cmd = Command::new(compiler_exe);
        cmd.args(flags.build());
        cmd.arg(source.as_path());

        Ok(Self {
            source: source.clone(),
            object: ObjectFilePath(object_path),
            dep_file,
            command: cmd,
        })
    }

    pub fn to_cache_entry(&self, hash: String) -> CacheEntry {
        CacheEntry {
            hash,
            object: self.object.0.clone(),
        }
    }
}
