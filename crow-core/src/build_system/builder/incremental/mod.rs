pub mod cache;

use super::*;
use crate::build_system::ToolchainExecutor;
use crate::config::PackageConfig;
use crate::utils;
use cache::*;
use crow_utils::LogLevel;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

pub struct IncrementalBuilder<'a> {
    base: &'a BuildSystem,
    build_dir: PathBuf,
    cache_path: PathBuf,
}

impl<'a> IncrementalBuilder<'a> {
    pub fn new(base: &'a BuildSystem) -> anyhow::Result<Self> {
        let build_dir = crow_utils::environment::Environment::build_dir().join(&base.profile_name);
        std::fs::create_dir_all(&build_dir)?;

        let cache_path = build_dir.join(format!("crow-{}.cache", base.profile_name));
        Ok(Self {
            base,
            build_dir,
            cache_path,
        })
    }

    pub fn build(
        &self,
        jobs: Option<usize>,
        package_config: &PackageConfig,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let old_cache =
            cache::BuildCache::load_cache(&self.cache_path, self.base.profile_config.incremental)?;
        let mut new_cache = cache::BuildCache::default();
        let sources = utils::find_source_files(package_config)?;

        let num_jobs = jobs.unwrap_or_else(|| {
            thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        });
        let pool = threadpool::ThreadPool::new(num_jobs);
        let (tx, rx) = mpsc::channel();
        let mut cache_updates: HashMap<String, (u64, u64, PathBuf)> = HashMap::new();

        for source_path in &sources {
            let obj_path = self
                .build_dir
                .join(source_path.with_extension("o").file_name().unwrap());
            let source_hash = xxhash_rust::xxh3::xxh3_64(&std::fs::read(source_path)?);

            let args = self.base.build_compile_args(source_path, &obj_path)?;
            let flags_hash =
                cache::BuildCache::compute_flags_hash(&self.base.toolchain.compiler, &args);

            let source_key = source_path.to_string_lossy().to_string();
            let mut need_compile = true;

            if self.base.profile_config.incremental {
                if let Some(entry) = old_cache.entries.get(&source_key) {
                    if entry.source_hash == source_hash
                        && entry.flags_hash == flags_hash
                        && Path::new(&entry.obj_path).exists()
                    {
                        let dep_path = Path::new(&entry.obj_path).with_extension("d");
                        if dep_path.exists() {
                            match cache::BuildCache::parse_dep_file(&dep_path)
                                .and_then(|deps| cache::BuildCache::compute_deps_hash(&deps))
                            {
                                Ok(current_deps_hash) if entry.deps_hash == current_deps_hash => {
                                    if self.base.logger.verbose {
                                        self.base.logger.log(
                                            LogLevel::Info,
                                            &format!("[CACHED] {}", source_path.display()),
                                            2,
                                        );
                                    }
                                    need_compile = false;
                                    new_cache.entries.insert(source_key.clone(), entry.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if !need_compile {
                tx.send((source_path.clone(), Ok((obj_path.clone(), 0))))
                    .unwrap();
                continue;
            }

            cache_updates.insert(
                source_key.clone(),
                (source_hash, flags_hash, obj_path.clone()),
            );

            let tx = tx.clone();
            let compiler_path = self.base.toolchain.compiler.clone();
            let source_clone = source_path.clone();
            let obj_path_clone = obj_path.clone();
            let incremental = self.base.profile_config.incremental;
            let verbose_clone = self.base.logger.verbose;
            let toolchain_clone = self.base.toolchain.clone();
            let profile_config_clone = self.base.profile_config.clone();
            let package_config_clone = package_config.clone();
            let downloaded_deps_paths_clone = self.base.downloaded_deps_paths.clone();
            let dep_build_outputs_clone = self.base.dep_build_outputs.clone();
            let logger_clone = self.base.logger.clone();

            pool.execute(move || {
                let args_for_thread = BuildSystem::build_compile_args_static(
                    &toolchain_clone,
                    &profile_config_clone,
                    &package_config_clone,
                    &downloaded_deps_paths_clone,
                    &dep_build_outputs_clone,
                    &source_clone,
                    &obj_path_clone,
                )
                .expect("Failed to build compile args in thread");

                let result = <BuildSystem as ToolchainExecutor>::compile_with_args(
                    &compiler_path,
                    &args_for_thread,
                    &source_clone,
                    &obj_path_clone,
                    incremental,
                    &logger_clone,
                );
                if verbose_clone {
                    if let Ok(_) = &result {
                        logger_clone.log(
                            LogLevel::Custom("\x1b[32m"),
                            &format!("[COMPILED] {}", source_clone.display()),
                            2,
                        );
                    }
                }
                tx.send((source_clone, result)).unwrap();
            });
        }

        drop(tx);
        let mut object_files = Vec::new();
        let mut had_errors = false;

        for (source, result) in rx.iter() {
            match result {
                Ok((obj_path, deps_hash)) => {
                    object_files.push(obj_path.clone());
                    if let Some((source_hash, flags_hash, _)) =
                        cache_updates.get(&source.to_string_lossy().to_string())
                    {
                        new_cache.entries.insert(
                            source.to_string_lossy().to_string(),
                            cache::CacheEntry {
                                source_hash: *source_hash,
                                flags_hash: *flags_hash,
                                deps_hash,
                                obj_path: obj_path.to_string_lossy().to_string(),
                            },
                        );
                    }
                }
                Err(_) => had_errors = true,
            }
        }

        if had_errors {
            anyhow::bail!("Compilation failed.");
        }

        if self.base.profile_config.incremental {
            cache::BuildCache::save_cache(&self.cache_path, &new_cache)?;
        }

        Ok(object_files)
    }
}
