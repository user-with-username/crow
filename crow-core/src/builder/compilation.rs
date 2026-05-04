use crate::builder::context::CompilationContext;
use crate::builder::incremental::IncrementalManager;
use crate::builder::paths::ObjectFilePath;
use crate::builder::tasks::database::CompilationDatabase;
use crate::project::Project;
use anyhow::Result;
use crow_utils::progress::ProgressBar;
use rayon::prelude::*;
use std::path::PathBuf;

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

    pub fn compile(&self, progress: Option<&ProgressBar>) -> Result<Vec<ObjectFilePath>> {
        self.project.create_dirs()?;

        let context = CompilationContext::new(self.compiler_exe, self.project, progress)?;
        let cache_path = self.project.profile_dir().join(format!(
            ".{}.fingerprint.json",
            self.project.package.output_stem()
        ));
        let cache_manager =
            IncrementalManager::new(&cache_path, self.project.profile.incremental());

        let sources = self.project.find_sources();

        let compile_task = |path: &PathBuf| -> Result<(ObjectFilePath, PathBuf, Vec<String>)> {
            context.compile_unit(path, &cache_manager)
        };

        let results: Result<Vec<(ObjectFilePath, PathBuf, Vec<String>)>> =
            if self.project.config.build.parallelism {
                sources.par_iter().map(compile_task).collect()
            } else {
                sources.iter().map(compile_task).collect()
            };

        match results {
            Ok(res) => {
                let mut db = CompilationDatabase::new();
                let mut objects = Vec::with_capacity(res.len());

                for (obj, src, args) in res {
                    db.add_entry(&self.project, &src, args);
                    objects.push(obj);
                }

                db.save(&self.project.root)?;
                cache_manager.finalize()?;
                Ok(objects)
            }
            Err(e) => {
                let _ = cache_manager.finalize();
                Err(e)
            }
        }
    }
}
