use super::{get_artifacts, link_name_from_library_file, Wheel, WheelArtifacts};
use crate::builder::kinds::compiler_kind::CompilerKind;
use anyhow::{Context, Result};
use crow_utils::find_executable;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct CmakeWheel {
    root: PathBuf,
}

impl CmakeWheel {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    fn get_cxx_flags(&self, compiler_flags: &[String], compiler_kind: CompilerKind) -> String {
        let mut flags = compiler_flags.to_vec();
        flags.retain(|f| {
            if compiler_kind.is_msvc() {
                f.starts_with('/') || f.starts_with("-D") || f.starts_with("-I")
            } else {
                f.starts_with('-')
            }
        });
        flags.join(" ")
    }

    fn write_file_api_query(build_dir: &Path) -> Result<()> {
        let query_dir = build_dir.join(".cmake/api/v1/query");
        fs::create_dir_all(&query_dir)?;
        fs::write(query_dir.join("codemodel-v2"), "")?;
        Ok(())
    }

    fn scan_all_target_files(build_dir: &Path) -> Result<Vec<TargetFile>> {
        let reply_dir = build_dir.join(".cmake/api/v1/reply");
        let mut targets = Vec::new();

        let entries = fs::read_dir(&reply_dir)
            .context("cmake File API reply dir not found — configure may have failed")?;

        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            if !name.starts_with("target-") || !name.ends_with(".json") {
                continue;
            }
            let text = match fs::read_to_string(entry.path()) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let target: TargetFile = match serde_json::from_str(&text) {
                Ok(t) => t,
                Err(_) => continue,
            };
            targets.push(target);
        }

        Ok(targets)
    }

    fn is_consumer_library_target(target: &TargetFile) -> bool {
        let is_library = matches!(
            target.r#type.as_str(),
            "STATIC_LIBRARY" | "SHARED_LIBRARY" | "MODULE_LIBRARY"
        );
        if !is_library || target.is_imported.unwrap_or(false) {
            return false;
        }
        let name = target.name.to_ascii_lowercase();
        if name.contains("test")
            || name.contains("gtest")
            || name.contains("mock")
            || name.contains("benchmark")
        {
            return false;
        }
        if let Some(artifacts) = &target.artifacts {
            for artifact in artifacts {
                if super::should_skip_library_path(Path::new(&artifact.path)) {
                    return false;
                }
            }
        }
        true
    }

    fn is_interface_library_target(target: &TargetFile) -> bool {
        if target.r#type != "INTERFACE_LIBRARY" {
            return false;
        }
        let name = target.name.to_ascii_lowercase();
        if name.contains("test")
            || name.contains("gtest")
            || name.contains("mock")
            || name.contains("benchmark")
            || name == "cmake_object_depends"
        {
            return false;
        }
        true
    }

    fn artifacts_from_built_targets(
        build_dir: &Path,
        target_names: &[String],
    ) -> Result<WheelArtifacts> {
        let mut artifacts = WheelArtifacts::default();

        for target in Self::scan_all_target_files(build_dir)? {
            if !target_names.iter().any(|n| n == &target.name) {
                continue;
            }
            let Some(artifact_paths) = &target.artifacts else {
                continue;
            };
            for artifact in artifact_paths {
                let artifact_path = build_dir.join(&artifact.path);
                if !artifact_path.is_file() {
                    continue;
                }
                let Some(lib_name) = link_name_from_library_file(&artifact_path) else {
                    continue;
                };
                let parent = artifact_path
                    .parent()
                    .context("library artifact has no parent directory")?
                    .to_path_buf();
                if !artifacts.lib_names.contains(&lib_name) {
                    artifacts.lib_names.push(lib_name);
                }
                if !artifacts.lib_paths.contains(&parent) {
                    artifacts.lib_paths.push(parent);
                }
            }
        }

        Ok(artifacts)
    }

    fn include_dirs_for_interface_targets(
        interface_targets: &[&TargetFile],
        root: &Path,
        install_dir: &Path,
    ) -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = Vec::new();

        let mut add = |p: PathBuf| {
            if p.is_dir() && !dirs.contains(&p) {
                dirs.push(p);
            }
        };

        for target in interface_targets {
            if let Some(src) = &target.source_dir {
                let src = PathBuf::from(src);
                add(src.join("include"));
                add(src.join("single_include"));
                add(src.clone());
            }
        }

        add(install_dir.join("include"));

        add(root.join("include"));
        add(root.join("single_include"));

        dirs
    }
}

#[derive(Deserialize)]
struct TargetFile {
    name: String,
    r#type: String,
    #[serde(rename = "isImported")]
    is_imported: Option<bool>,
    #[serde(rename = "sourceDir")]
    source_dir: Option<String>,
    artifacts: Option<Vec<TargetArtifact>>,
}

#[derive(Deserialize)]
struct TargetArtifact {
    path: String,
}

impl Wheel for CmakeWheel {
    fn detects(&self, root: &Path) -> bool {
        root.join("CMakeLists.txt").exists()
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn build(
        &self,
        build_dir: &Path,
        profile: &str,
        compiler_flags: &[String],
        build_flags: &[String],
        compiler_path: Option<&str>,
        compiler_kind: CompilerKind,
    ) -> Result<WheelArtifacts> {
        let out_dir = build_dir.join("cmake_wheel");
        fs::create_dir_all(&out_dir)?;

        let build_type = match profile {
            "release" => "Release",
            _ => "Debug",
        };

        let cxx_flags = self.get_cxx_flags(compiler_flags, compiler_kind);

        Self::write_file_api_query(&out_dir)?;

        let cmake_exe = find_executable("cmake")?;

        let mut cmd = Command::new(&cmake_exe);

        #[cfg(target_os = "windows")]
        if !compiler_kind.is_msvc() {
            if find_executable("ninja").is_ok() {
                cmd.arg("-G").arg("Ninja");
            } else {
                cmd.arg("-G").arg("MinGW Makefiles");
            }
        }

        cmd.arg("-B")
            .arg(&out_dir)
            .arg("-S")
            .arg(&self.root)
            .arg(format!("-DCMAKE_BUILD_TYPE={}", build_type))
            .arg("-DCMAKE_INSTALL_PREFIX=install")
            .arg("-DBUILD_SHARED_LIBS=OFF")
            .arg("-DBUILD_TESTING=OFF")
            .arg("-DCMAKE_CXX_EXTENSIONS=OFF");

        if let Some(path) = compiler_path {
            cmd.arg(format!("-DCMAKE_CXX_COMPILER={}", path));
        }

        for flag in build_flags {
            cmd.arg(format!(
                "-D{}",
                flag.trim_start_matches("-D").trim_start_matches('-')
            ));
        }

        if !cxx_flags.is_empty() {
            cmd.arg(format!("-DCMAKE_CXX_FLAGS={}", cxx_flags));
            cmd.arg(format!(
                "-DCMAKE_CXX_FLAGS_{}={}",
                build_type.to_uppercase(),
                cxx_flags
            ));
        }

        #[cfg(target_os = "windows")]
        if compiler_kind.is_msvc() {
            let runtime = match build_type {
                "Debug" => "MultiThreadedDebugDLL",
                _ => "MultiThreadedDLL",
            };
            cmd.arg(format!("-DCMAKE_MSVC_RUNTIME_LIBRARY={}", runtime));
            cmd.arg("-DCMAKE_CXX_FLAGS_INIT=");
        }

        cmd.stdout(Stdio::null())
            .stderr(Stdio::piped())
            .current_dir(&self.root);

        let output = cmd.output().context("failed to run cmake configure")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cmake configure failed:\n{}", stderr);
        }

        let all_targets =
            Self::scan_all_target_files(&out_dir).context("failed to read cmake File API reply")?;

        let library_target_names: Vec<String> = all_targets
            .iter()
            .filter(|t| Self::is_consumer_library_target(t))
            .map(|t| t.name.clone())
            .collect();

        let interface_targets: Vec<&TargetFile> = all_targets
            .iter()
            .filter(|t| Self::is_interface_library_target(t))
            .collect();

        let is_header_only = library_target_names.is_empty() && !interface_targets.is_empty();
        let nothing_found = library_target_names.is_empty() && interface_targets.is_empty();

        for target in &library_target_names {
            let output = Command::new(&cmake_exe)
                .arg("--build")
                .arg(&out_dir)
                .arg("--config")
                .arg(build_type)
                .arg("--target")
                .arg(target)
                .arg("--parallel")
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .current_dir(&self.root)
                .output()
                .with_context(|| format!("failed to run cmake build for target '{}'", target))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("cmake build of target '{}' failed:\n{}", target, stderr);
            }
        }

        let install_dir = out_dir.join("install");
        let _ = Command::new(&cmake_exe)
            .arg("--install")
            .arg(&out_dir)
            .arg("--config")
            .arg(build_type)
            .arg("--prefix")
            .arg(&install_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .current_dir(&self.root)
            .output()
            .ok();

        let mut artifacts = if is_header_only || nothing_found {
            WheelArtifacts::default()
        } else {
            let mut arts = Self::artifacts_from_built_targets(&out_dir, &library_target_names)?;

            if arts.lib_names.is_empty() {
                arts = get_artifacts(
                    &self.root,
                    &out_dir,
                    vec![out_dir.join("generated"), install_dir.join("include")],
                    vec![
                        install_dir.join("lib"),
                        out_dir.clone(),
                        out_dir.join(build_type),
                        out_dir.join(build_type.to_lowercase()),
                        out_dir.join("lib"),
                    ],
                    4,
                )?;
            } else {
                let include_arts = get_artifacts(
                    &self.root,
                    &out_dir,
                    vec![out_dir.join("generated"), install_dir.join("include")],
                    vec![install_dir.clone()],
                    4,
                )?;
                for dir in include_arts.include_dirs {
                    if !arts.include_dirs.contains(&dir) {
                        arts.include_dirs.push(dir);
                    }
                }
            }

            arts
        };

        if !interface_targets.is_empty() {
            for dir in Self::include_dirs_for_interface_targets(
                &interface_targets,
                &self.root,
                &install_dir,
            ) {
                if !artifacts.include_dirs.contains(&dir) {
                    artifacts.include_dirs.push(dir);
                }
            }
        }

        if artifacts.include_dirs.is_empty() {
            for candidate in &[
                self.root.join("include"),
                self.root.join("single_include"),
                self.root.to_path_buf(),
                install_dir.join("include"),
            ] {
                if candidate.is_dir() && !artifacts.include_dirs.contains(candidate) {
                    artifacts.include_dirs.push(candidate.clone());
                }
            }
        }

        Ok(artifacts)
    }
}
