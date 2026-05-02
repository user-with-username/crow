use super::{Wheel, WheelArtifacts, get_artifacts};
use anyhow::{Context, Result};
use crow_utils::find_executable;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct CmakeWheel {
    root: PathBuf,
}

impl CmakeWheel {
    pub fn new(root: &Path) -> Self {
        Self { root: root.to_path_buf() }
    }

    fn get_cxx_flags(&self, compiler_flags: &[String]) -> String {
        let mut flags = Vec::new();
        flags.extend(compiler_flags.iter().map(|f| f.as_str()));
        flags.join(" ")
    }
}

impl Wheel for CmakeWheel {
    fn detects(&self, root: &Path) -> bool {
        root.join("CMakeLists.txt").exists()
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn build(&self, build_dir: &Path, profile: &str, compiler_flags: &[String], build_flags: &[String]) -> Result<WheelArtifacts> {
        let out_dir = build_dir.join("cmake_wheel");
        fs::create_dir_all(&out_dir)?;

        let build_type = match profile {
            "release" => "Release",
            _ => "Debug",
        };

        let cxx_flags = self.get_cxx_flags(compiler_flags);

        let cmake_exe = find_executable("cmake")?;
        let mut cmd = Command::new(&cmake_exe);
        cmd.arg("-B").arg(&out_dir)
            .arg("-S").arg(&self.root)
            .arg(format!("-DCMAKE_BUILD_TYPE={}", build_type))
            .arg("-DCMAKE_INSTALL_PREFIX=install")
            .arg("-DBUILD_TESTING=OFF")
            .arg("-DBUILD_EXAMPLES=OFF")
            .arg("-DBUILD_SHARED_LIBS=OFF")
            .arg("-DCMAKE_CXX_EXTENSIONS=OFF")
            .arg("-DBUILD_TESTS=OFF")
            .arg("-DGTEST_BUILD=OFF")
            .arg("-DGMOCK_BUILD=OFF")
            .arg("-DBUILD_GMOCK=OFF")
            .arg("-DBUILD_GTEST=OFF")
            .arg("-DINSTALL_GTEST=OFF")
            ;
        // Pass build_flags as direct CMake options (-D...)
        for flag in build_flags {
            cmd.arg(format!("-D{}", flag.trim_start_matches("-D").trim_start_matches("-")));
        }
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null())
            .current_dir(&self.root);

        if !cxx_flags.is_empty() {
            cmd.arg(format!("-DCMAKE_CXX_FLAGS={}", cxx_flags));
            cmd.arg(format!("-DCMAKE_CXX_FLAGS_{}={}", build_type.to_uppercase(), cxx_flags));
        }

        #[cfg(target_os = "windows")]
        {
            cmd.arg("-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL");
            cmd.arg("-DCMAKE_CXX_FLAGS_INIT=");
        }

        let status = cmd.status()
            .context("failed to run cmake configure")?;
        if !status.success() {
            anyhow::bail!("cmake configure failed");
        }

        let status = Command::new(&cmake_exe)
            .arg("--build").arg(&out_dir)
            .arg("--config").arg(build_type)
            .arg("--parallel")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .current_dir(&self.root)
            .status()
            .context("failed to run cmake build")?;
        if !status.success() {
            anyhow::bail!("cmake build failed");
        }

        let _ = Command::new(&cmake_exe)
            .arg("--install").arg(&out_dir)
            .arg("--config").arg(build_type)
            .arg("--prefix").arg(out_dir.join("install"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .current_dir(&self.root)
            .status();

        let install_dir = out_dir.join("install");

        let artifacts = get_artifacts(
            &self.root,
            &out_dir,
            vec![out_dir.join("generated"), install_dir.join("include")],
            vec![
                out_dir.clone(),
                out_dir.join(&build_type),
                out_dir.join(build_type.to_lowercase()),
                out_dir.join("lib"),
                install_dir.join("lib"),
            ],
            4,
        )?;

        Ok(artifacts)
    }
}
