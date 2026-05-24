use crate::builder::incremental::hash_files;
use crate::builder::linking::{DynamicLinkBuilder, LinkingBuilder};
use crate::builder::paths::ObjectFilePath;
use crate::builder::CompilationBuilder;
use crate::project::Project;
use anyhowed::{Context, Result};
use crow_utils::{find_executable, status};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

struct TestCase {
    source: PathBuf,
    display_name: String,
    executable: PathBuf,
}

enum RunKind {
    Test,
    Bench,
}

impl RunKind {
    fn name(&self) -> &'static str {
        match self {
            RunKind::Test => "test",
            RunKind::Bench => "benchmark",
        }
    }

    fn singular(&self) -> &'static str {
        match self {
            RunKind::Test => "test",
            RunKind::Bench => "benchmark",
        }
    }
}

impl Project {
    pub fn test(&self, trailing: &[String]) -> Result<()> {
        self.run_tests_or_benches(RunKind::Test, trailing)
    }

    pub fn bench(&self, trailing: &[String]) -> Result<()> {
        self.run_tests_or_benches(RunKind::Bench, trailing)
    }

    pub fn fmt(&self) -> Result<()> {
        let all_files = self.find_all();
        let formatter = find_executable(&self.config.build.formatter)?;
        for file in all_files {
            Command::new(formatter.clone()).arg("-i").arg(file.clone()).status()?;
        }

        Ok(())

    }

    fn run_tests_or_benches(&self, kind: RunKind, trailing: &[String]) -> Result<()> {
        let start_time = Instant::now();

        let sources = match kind {
            RunKind::Test => self.find_tests(),
            RunKind::Bench => self.find_benches(),
        };

        if sources.is_empty() {
            status!("No", "{}s found", kind.name());
            return Ok(());
        }

        let cases: Vec<TestCase> = sources
            .iter()
            .map(|source| {
                Ok(TestCase {
                    display_name: self.make_display_name(source),
                    executable: self.make_executable_path(source)?,
                    source: source.clone(),
                })
            })
            .collect::<Result<_>>()?;

        let project_objects = self.compile_project_objects_for_tests_or_benches()?;

        let mut passed = 0usize;
        let mut failed = 0usize;

        for case in &cases {
            status!(
                "     Running",
                "{} ({})",
                case.source
                    .strip_prefix(&self.root)
                    .unwrap_or(&case.source)
                    .display(),
                case.executable.display()
            );

            let source_objects = self.compile_source(&case.source)?;

            self.link_executable(&case.executable, &project_objects, &source_objects, &kind)?;

            print!("\nrunning 1 {}\n", kind.singular());
            print!("{} {} ... ", kind.singular(), case.display_name);

            let status = {
                let mut cmd = Command::new(&case.executable);
                if !trailing.is_empty() {
                    cmd.args(trailing);
                }
                cmd.status()
                    .with_context(|| format!("failed to run {}", case.executable.display()))?
            };

            if status.success() {
                println!("\x1b[32mok\x1b[0m");
                passed += 1;
            } else {
                println!("\x1b[31mFAILED\x1b[0m");
                failed += 1;
            }
        }

        println!();

        let duration = start_time.elapsed();
        let status_text = if failed == 0 { "ok" } else { "FAILED" };
        let status_color = if failed == 0 {
            "\x1b[32mok\x1b[0m"
        } else {
            "\x1b[31mFAILED\x1b[0m"
        };

        println!(
            "{} result: {}. {} passed; {} failed; finished in {:.2}s",
            kind.name(),
            status_color,
            passed,
            failed,
            duration.as_secs_f64()
        );

        if failed > 0 {
            anyhowed::bail!("{} failed, {} passed", status_text, passed);
        }

        Ok(())
    }

    fn make_display_name(&self, source: &Path) -> String {
        let relative = source.strip_prefix(&self.root).unwrap_or(source);
        let parent = relative.parent().unwrap_or_else(|| Path::new(""));
        let module = parent
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("::");
        let stem = source
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());

        if module.is_empty() {
            stem
        } else {
            format!("{module}::{stem}")
        }
    }

    fn make_executable_path(&self, source: &Path) -> Result<PathBuf> {
        let stem = source
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .context("source has no file name")?;
        let hash = hash_files(&[source.to_path_buf()])?;
        let suffix = &hash[..16.min(hash.len())];
        let ext = if cfg!(target_os = "windows") {
            "exe"
        } else {
            ""
        };
        let name = if ext.is_empty() {
            format!("{stem}-{suffix}")
        } else {
            format!("{stem}-{suffix}.{ext}")
        };
        Ok(self.profile_dir().join("deps").join(name))
    }

    fn compile_project_objects_for_tests_or_benches(&self) -> Result<Vec<ObjectFilePath>> {
        if self.package.r#type.is_static() {
            return Ok(Vec::new());
        }

        let entry_points = self.package.r#type.entry_points();
        let sources: Vec<PathBuf> = self
            .find_sources()
            .into_iter()
            .filter(|path| !Self::is_entry_source(path, &entry_points))
            .collect();

        if sources.is_empty() {
            return Ok(Vec::new());
        }

        let compiler_exe = self.compiler_exe_string();
        CompilationBuilder::new(&compiler_exe, self).compile_paths(&sources, None)
    }

    fn compile_source(&self, source: &Path) -> Result<Vec<ObjectFilePath>> {
        let compiler_exe = self.compiler_exe_string();
        CompilationBuilder::new(&compiler_exe, self).compile_paths(&[source.to_path_buf()], None)
    }

    fn compiler_exe_string(&self) -> String {
        self.config
            .build
            .compiler
            .path()
            .cloned()
            .unwrap_or_else(|| self.compiler_path().to_string())
    }

    fn is_entry_source(source: &Path, entry_points: &[String]) -> bool {
        source
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| entry_points.iter().any(|entry| entry == name))
    }

    fn link_executable(
        &self,
        output: &Path,
        project_objects: &[ObjectFilePath],
        source_objects: &[ObjectFilePath],
        kind: &RunKind,
    ) -> Result<()> {
        let linker_exe = self
            .config
            .build
            .linker
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.linker_path());
        let archiver_exe = self
            .config
            .build
            .archiver
            .path()
            .map(|p| p.as_str())
            .unwrap_or_else(|| self.archiver_path());

        let linker = LinkingBuilder::new(linker_exe, archiver_exe, self, &[], None);

        let mut link_objects: Vec<ObjectFilePath> = project_objects.to_vec();
        link_objects.extend_from_slice(source_objects);

        let mut extra_inputs = Vec::new();
        if self.package.r#type.is_static() {
            let lib_path = self.output_path();
            if !lib_path.exists() {
                anyhowed::bail!(
                    "Library '{}' not found. Build the project before running {}s.",
                    lib_path.display(),
                    kind.name()
                );
            }
            extra_inputs.push(lib_path.to_string_lossy().into_owned());
        }

        DynamicLinkBuilder::new(&linker).build_executable(
            output,
            &link_objects,
            &extra_inputs,
            false,
        )
    }
}
