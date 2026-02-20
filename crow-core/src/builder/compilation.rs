use crate::builder::context::CompilationContext;
use crate::builder::incremental::IncrementalManager;
use crate::builder::paths::ObjectFilePath;
use crate::builder::tasks::source::CompileCommand;
use crate::project::Project;
use anyhow::Result;
use crow_utils::ProgressBar;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;

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
        let context = CompilationContext::new(self.compiler_exe, self.project)?;

        let cache_path = self.project.profile_dir().join(".fingerprint.json");
        let cache_manager =
            IncrementalManager::new(&cache_path, self.project.profile.incremental());

        let sources: Vec<_> = self.project.find_sources().into_iter().collect();
        let progress = ProgressBar::new(
            sources.len(),
            format!(
                "{}({})",
                self.project.config.r#type.display_name(&self.project.config.package.name),
                self.project.config.r#type.type_str()
            ),
        );

        let results: Result<Vec<(ObjectFilePath, CompileCommand)>> = if self.project.config.build.parallelism {
            sources
                .par_iter()
                .map(|path| context.compile_unit(path, &cache_manager, &progress))
                .collect()
        } else {
            sources
                .iter()
                .map(|path| context.compile_unit(path, &cache_manager, &progress))
                .collect()
        };

        let results = results?;
        let (objects, commands): (Vec<_>, Vec<_>) = results.into_iter().unzip();

        let json_path = self.project.root.join("compile_commands.json");
        let mut file = File::create(json_path)?;
        let json_data = serde_json::to_string_pretty(&commands)?;
        file.write_all(json_data.as_bytes())?;

        progress.finish();
        cache_manager.finalize()?;

        Ok(objects)
    }
}