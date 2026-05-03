use super::{get_artifacts, Wheel, WheelArtifacts};
use anyhow::{Context, Result};
use crow_utils::find_executable;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct BazelWheel {
    root: PathBuf,
}

impl BazelWheel {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    fn get_bazel_args(&self, compiler_flags: &[String], build_flags: &[String]) -> Vec<String> {
        let mut args = Vec::new();

        if !compiler_flags.is_empty() {
            let flags_str = compiler_flags.join(" ");
            args.push(format!("--cxxopt={}", flags_str));
            args.push(format!("--copt={}", flags_str));
        }

        // Pass build_flags as --define options to Bazel
        for flag in build_flags {
            args.push(format!(
                "--define={}",
                flag.trim_start_matches("-D").trim_start_matches("-")
            ));
        }

        args
    }
}

impl Wheel for BazelWheel {
    fn detects(&self, root: &Path) -> bool {
        root.join("WORKSPACE").exists() || root.join("WORKSPACE.bazel").exists()
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
        let out_dir = build_dir.join("bazel_wheel");
        fs::create_dir_all(&out_dir)?;

        let compilation_mode = match profile {
            "release" => "opt",
            _ => "dbg",
        };

        let bazel_exe = find_executable("bazel")?;
        let mut cmd = Command::new(&bazel_exe);
        cmd.arg("build")
            .arg("//")
            .arg(format!("--compilation_mode={}", compilation_mode))
            .arg("--symlink_prefix=")
            .arg(format!("--output_base={}", out_dir.display()))
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped());

        for arg in self.get_bazel_args(compiler_flags, build_flags) {
            cmd.arg(arg);
        }

        let output = cmd
            .current_dir(&self.root)
            .output()
            .context("failed to run bazel build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("bazel build failed:\n{}", stderr);
        }

        let bazel_bin = out_dir
            .join("execroot")
            .join(
                self.root
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref(),
            )
            .join("bazel-out");

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else {
            "aarch64"
        };
        let platform = if cfg!(target_os = "macos") {
            format!("darwin_{}", arch)
        } else if cfg!(windows) {
            "x64_windows".to_string()
        } else {
            format!("k8-{}", compilation_mode)
        };

        let bazel_out = bazel_bin.join(&platform).join("bin");

        let artifacts = get_artifacts(
            &self.root,
            &out_dir,
            vec![bazel_bin.join("include")],
            vec![bazel_out],
            4,
        )?;

        Ok(artifacts)
    }
}
