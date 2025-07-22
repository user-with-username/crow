use crate::build_system::BuildSystem;
use anyhow::Context;
use crow_utils::logger::{LogLevel, Logger};
use std::{
    path::Path,
    process::{Command, Stdio},
};

pub trait GitManager {
    fn check_git_available() -> anyhow::Result<()>;
    fn git_clone(
        repo_url: &str,
        branch: &str,
        dest_path: &Path,
        verbose: bool,
        logger: &Logger,
    ) -> anyhow::Result<()>;
    fn git_pull(repo_path: &Path, verbose: bool, logger: &Logger) -> anyhow::Result<()>;
}

impl GitManager for BuildSystem {
    fn check_git_available() -> anyhow::Result<()> {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .context("`git` not found. Please install Git.")?;
        Ok(())
    }

    fn git_clone(
        repo_url: &str,
        branch: &str,
        dest_path: &Path,
        verbose: bool,
        logger: &Logger,
    ) -> anyhow::Result<()> {
        let mut command = Command::new("git");
        command.arg("clone").arg("--depth").arg("1");

        if !branch.is_empty() {
            command.arg("--branch").arg(branch);
        }

        command.arg(repo_url).arg(dest_path);
        command.stderr(Stdio::piped());
        command.stdout(Stdio::piped());

        let output = command.output().context(format!(
            "Failed while executing `git clone` for '{}'",
            repo_url
        ))?;

        if !output.status.success() {
            let stdout_output = String::from_utf8_lossy(&output.stdout);
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            logger.log(
                LogLevel::Error,
                format!(
                    "Git clone failed for '{}':\n{} {}",
                    repo_url, stdout_output, stderr_output
                ),
                0,
            );
            anyhow::bail!("Git clone failed for '{}'", repo_url);
        } else if verbose {
            let stdout_output = String::from_utf8_lossy(&output.stdout);
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            if !stdout_output.is_empty() {
                logger.log(LogLevel::Dim, "Git clone stdout:", 2);
                logger.log((), stdout_output, 2);
            }
            if !stderr_output.is_empty() {
                logger.log(LogLevel::Dim, "Git clone stderr:", 2);
                logger.log((), stderr_output, 2);
            }
        }
        Ok(())
    }

    fn git_pull(repo_path: &Path, verbose: bool, logger: &Logger) -> anyhow::Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(repo_path).arg("pull");
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let output = cmd.output()?;
        if !output.status.success() {
            let stdout_output = String::from_utf8_lossy(&output.stdout);
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            logger.log(
                LogLevel::Error,
                format!(
                    "Failed while pulling updates for '{}':\n{} {}",
                    repo_path.display(),
                    stdout_output,
                    stderr_output
                ),
                0,
            );
            anyhow::bail!("Failed while pulling updates for '{}'", repo_path.display());
        } else if verbose {
            let stdout_output = String::from_utf8_lossy(&output.stdout);
            let stderr_output = String::from_utf8_lossy(&output.stderr);
            if !stdout_output.is_empty() {
                logger.log(LogLevel::Dim, "Git pull stdout:", 2);
                logger.log((), stdout_output, 2);
            }
            if !stderr_output.is_empty() {
                logger.log(LogLevel::Dim, "Git pull stderr:", 2);
                logger.log((), stderr_output, 2);
            }
        }
        Ok(())
    }
}
