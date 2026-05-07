use super::{get_artifacts, Wheel, WheelArtifacts};
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

    fn get_cxx_flags(&self, compiler_flags: &[String]) -> String {
        compiler_flags.join(" ")
    }

    fn write_file_api_query(build_dir: &Path) -> Result<()> {
        let query_dir = build_dir.join(".cmake/api/v1/query");
        fs::create_dir_all(&query_dir)?;
        fs::write(query_dir.join("codemodel-v2"), "")?;
        Ok(())
    }

    fn read_library_targets(build_dir: &Path) -> Result<Vec<String>> {
        let reply_dir = build_dir.join(".cmake/api/v1/reply");

        let index_path = fs::read_dir(&reply_dir)
            .context("cmake File API reply dir not found — configure may have failed")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("index-") && n.ends_with(".json"))
                    .unwrap_or(false)
            })
            .context("cmake File API index file not found")?;

        let index_text = fs::read_to_string(&index_path)?;
        let index: IndexFile = serde_json::from_str(&index_text)
            .with_context(|| {
                format!(
                    "failed to parse cmake File API index:\n{}",
                    &index_text[..index_text.len().min(2000)]
                )
            })?;

        let codemodel_ref = index
            .objects
            .iter()
            .find(|o| o.kind == "codemodel")
            .context("no codemodel object found in cmake File API index")?;

        let codemodel_path = reply_dir.join(&codemodel_ref.json_file);
        let codemodel_text = fs::read_to_string(&codemodel_path)
            .with_context(|| format!("failed to read codemodel file: {}", codemodel_path.display()))?;
        let codemodel: Codemodel = serde_json::from_str(&codemodel_text)
            .context("failed to parse cmake codemodel")?;

        let mut library_targets = Vec::new();

        for config in &codemodel.configurations {
            for target_ref in &config.targets {
                let target_path = reply_dir.join(&target_ref.json_file);
                let target_text = match fs::read_to_string(&target_path) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                let target: TargetFile = match serde_json::from_str(&target_text) {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                let is_library = matches!(
                    target.r#type.as_str(),
                    "STATIC_LIBRARY" | "SHARED_LIBRARY" | "MODULE_LIBRARY" | "OBJECT_LIBRARY"
                );
                let is_imported = target.is_imported.unwrap_or(false);

                if is_library && !is_imported {
                    library_targets.push(target.name.clone());
                }
            }
        }

        library_targets.dedup();

        Ok(library_targets)
    }
}

#[derive(Deserialize)]
struct IndexFile {
    objects: Vec<IndexObject>,
}

#[derive(Deserialize)]
struct IndexObject {
    kind: String,
    #[serde(rename = "jsonFile")]
    json_file: String,
}

#[derive(Deserialize)]
struct Codemodel {
    configurations: Vec<CodemodelConfig>,
}

#[derive(Deserialize)]
struct CodemodelConfig {
    targets: Vec<TargetRef>,
}

#[derive(Deserialize)]
struct TargetRef {
    #[serde(rename = "jsonFile")]
    json_file: String,
}

#[derive(Deserialize)]
struct TargetFile {
    name: String,
    r#type: String,
    #[serde(rename = "isImported")]
    is_imported: Option<bool>,
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
    ) -> Result<WheelArtifacts> {
        let out_dir = build_dir.join("cmake_wheel");
        fs::create_dir_all(&out_dir)?;

        let build_type = match profile {
            "release" => "Release",
            _ => "Debug",
        };

        let cxx_flags = self.get_cxx_flags(compiler_flags);

        Self::write_file_api_query(&out_dir)?;

        let cmake_exe = find_executable("cmake")?;

        let mut cmd = Command::new(&cmake_exe);
        cmd.arg("-B")
            .arg(&out_dir)
            .arg("-S")
            .arg(&self.root)
            .arg(format!("-DCMAKE_BUILD_TYPE={}", build_type))
            .arg("-DCMAKE_INSTALL_PREFIX=install")
            .arg("-DBUILD_SHARED_LIBS=OFF")
            .arg("-DCMAKE_CXX_EXTENSIONS=OFF");

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
        {
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

        let library_targets = Self::read_library_targets(&out_dir)
            .context("failed to read cmake File API reply")?;

        if library_targets.is_empty() {
            anyhow::bail!(
                "cmake File API found no STATIC/SHARED/MODULE library targets in {}",
                self.root.display()
            );
        }

        for target in &library_targets {
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

        let _ = Command::new(&cmake_exe)
            .arg("--install")
            .arg(&out_dir)
            .arg("--config")
            .arg(build_type)
            .arg("--prefix")
            .arg(out_dir.join("install"))
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .current_dir(&self.root)
            .output()
            .ok();

        let install_dir = out_dir.join("install");

        let artifacts = get_artifacts(
            &self.root,
            &out_dir,
            vec![out_dir.join("generated"), install_dir.join("include")],
            vec![
                out_dir.clone(),
                out_dir.join(build_type),
                out_dir.join(build_type.to_lowercase()),
                out_dir.join("lib"),
                install_dir.join("lib"),
            ],
            4,
        )?;

        Ok(artifacts)
    }
}