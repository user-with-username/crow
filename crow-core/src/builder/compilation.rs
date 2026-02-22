use crate::builder::context::CompilationContext;
use crate::builder::incremental::IncrementalManager;
use crate::builder::paths::ObjectFilePath;
use crate::builder::tasks::database::CompilationDatabase;
use crate::project::Project;
use anyhow::Result;
use crow_utils::ProgressBar;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
        let context = CompilationContext::new(self.compiler_exe, self.project)?;

        let cache_path = self.project.profile_dir().join(".fingerprint.json");
        let cache_manager = IncrementalManager::new(&cache_path, self.project.profile.incremental());

        let sources: Vec<_> = self.project.find_sources().into_iter().collect();
        let progress = ProgressBar::new(
            sources.len(),
            format!("{}({})", 
                self.project.config.r#type.display_name(&self.project.config.package.name),
                self.project.config.r#type.type_str()
            ),
        );

        let failed = Arc::new(AtomicBool::new(false));

        let results: Result<Vec<(ObjectFilePath, std::path::PathBuf, Vec<String>)>> = if self.project.config.build.parallelism {
            sources.par_iter().map(|path| {
                if failed.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Aborted"));
                }
                let res = context.compile_unit(path, &cache_manager, &progress, Arc::clone(&failed));
                if res.is_err() { failed.store(true, Ordering::SeqCst); }
                res
            }).collect()
        } else {
            sources.iter().map(|path| {
                let res = context.compile_unit(path, &cache_manager, &progress, Arc::clone(&failed));
                if res.is_err() { failed.store(true, Ordering::SeqCst); }
                res
            }).collect()
        };

        progress.finish();

        match results {
            Ok(res) => {
                let mut db = CompilationDatabase::new();
                let mut objects = Vec::new();

                for (obj, src, args) in res {
                    db.add_entry(&self.project.root, &src, args);
                    objects.push(obj);
                }

                db.save(&self.project.root)?;
                cache_manager.finalize()?;
                Ok(objects)
            }
            Err(e) => {
                cache_manager.finalize()?;
                Err(e)
            }
        }
    }
}