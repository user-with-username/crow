use crate::builder::{flags::Flags, incremental::CacheEntry, paths::ObjectFileNaming};
use crate::builder::paths::{DependencyFilePath, DependencyFileNaming, ObjectFilePath, SourceFilePath};
use anyhow::{Context, Result};
use std::{
    path::Path,
    process::Command,
};

#[derive(Debug)]
pub struct SourceCompilationTask {
    pub source: SourceFilePath,
    pub object: ObjectFilePath,
    pub dep_file: DependencyFilePath,
    pub command: Command,
}

impl SourceCompilationTask {
    pub fn prepare(
        source: &SourceFilePath,
        compiler_exe: &str,
        build_config: &crate::config::BuildConfig,
        deps_dir: &Path,
        project: &crate::project::Project,
    ) -> Result<Self> {
        let compiler_kind = project.compiler_kind();

        let mut flags = Flags::new(compiler_kind);

        flags.compile_only();

        flags.standard_flags(project.release);

        for inc in &build_config.include_dirs {
            let inc_path = inc.to_string_lossy();
            flags.include_path(inc_path.into_owned());
        }

        for raw_flag in &build_config.flags {
            flags.add_raw(raw_flag.clone());
        }

        let source_file_stem = source
            .as_path()
            .file_stem()
            .context("Invalid file name")?
            .to_string_lossy();

        let dep_file = DependencyFileNaming::generate(deps_dir, source);
        flags.dependency_info(dep_file.as_path().to_string_lossy().into_owned());

        let obj_dir = if deps_dir.ends_with("deps") {
            deps_dir.parent().unwrap_or(deps_dir)
        } else {
            deps_dir
        };

        let object_path = ObjectFileNaming::generate(obj_dir, source, compiler_kind);

        flags.object_output(object_path.to_string_lossy().replace("\\", "/"));

        if compiler_kind.is_msvc() {
            let pdb_dir = obj_dir.join("pdb");
            std::fs::create_dir_all(&pdb_dir)?;
            let pdb_path = pdb_dir.join(format!("{}.pdb", source_file_stem));
            flags.program_database(pdb_path.to_string_lossy().replace("\\", "/"));
        }

        let mut cmd = Command::new(compiler_exe);

        let mut all_args = flags.build();

        let source_path = source.as_path().to_string_lossy().replace("\\", "/");
        all_args.push(source_path.clone());

        cmd.args(&all_args);
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