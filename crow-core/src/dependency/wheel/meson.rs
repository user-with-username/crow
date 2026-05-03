use super::{Wheel, WheelArtifacts, get_artifacts};
use anyhow::{Context, Result};
use crow_utils::find_executable;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct MesonWheel {
    root: PathBuf,
}

impl MesonWheel {
    pub fn new(root: &Path) -> Self {
        Self { root: root.to_path_buf() }
    }

    fn get_meson_args(&self, compiler_flags: &[String], build_flags: &[String]) -> Vec<String> {
        let mut args = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let mut cpp_flags = compiler_flags.to_vec();
            let mut c_flags = compiler_flags.to_vec();

            let has_utf8 = cpp_flags.iter().any(|f| f.contains("/utf-8"))
                || c_flags.iter().any(|f| f.contains("/utf-8"));

            if !has_utf8 {
                cpp_flags.push("/utf-8".to_string());
                c_flags.push("/utf-8".to_string());
            }

            if !cpp_flags.is_empty() {
                let flags_str = cpp_flags.join(" ");
                args.push(format!("-Dcpp_args={}", flags_str));
            }

            if !c_flags.is_empty() {
                let flags_str = c_flags.join(" ");
                args.push(format!("-Dc_args={}", flags_str));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if !compiler_flags.is_empty() {
                let flags_str = compiler_flags.join(" ");
                args.push(format!("-Dcpp_args={}", flags_str));
                args.push(format!("-Dc_args={}", flags_str));
            }
        }

        // Pass build_flags as direct Meson options (-D...)
        for flag in build_flags {
            args.push(format!("-D{}", flag.trim_start_matches("-D").trim_start_matches("-")));
        }

        args
    }
}

impl Wheel for MesonWheel {
    fn detects(&self, root: &Path) -> bool {
        root.join("meson.build").exists()
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn build(&self, build_dir: &Path, profile: &str, compiler_flags: &[String], build_flags: &[String]) -> Result<WheelArtifacts> {
        let out_dir = build_dir.join("meson_wheel");
        fs::create_dir_all(&out_dir)?;

        let build_type = match profile {
            "release" => "release",
            _ => "debug",
        };

        let meson_exe = find_executable("meson")?;
        let mut cmd = Command::new(&meson_exe);
        cmd.arg("setup")
            .arg(&out_dir)
            .arg(&self.root)
            .arg("--buildtype")
            .arg(build_type)
            .arg("-Dtests=false")
            .arg("-Dbenchmarks=false")
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped());
        
        for arg in self.get_meson_args(compiler_flags, build_flags) {
            cmd.arg(arg);
        }
        
        let output = cmd
            .current_dir(&self.root)
            .output()
            .context("failed to run meson setup")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("meson setup failed:\n{}", stderr);
        }

        let ninja_exe = find_executable("ninja")?;
        let output = Command::new(&ninja_exe)
            .arg("-C")
            .arg(&out_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
            .output()
            .context("failed to run ninja")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ninja build failed:\n{}", stderr);
        }

        let artifacts = get_artifacts(
            &self.root,
            &out_dir,
            vec![],
            vec![
                out_dir.clone(),
                out_dir.join("lib"),
                out_dir.join(&build_type),
            ],
            3,
        )?;

        Ok(artifacts)
    }
}