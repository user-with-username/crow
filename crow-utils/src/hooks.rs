use anyhow::{Context, Result};
use std::{path::Path, process::Command};

pub fn run_hooks(manifest_dir: &Path, hooks: &[String]) -> Result<()> {
    for hook in hooks {
        let args = shlex::split(hook)
            .ok_or_else(|| anyhow::anyhow!("failed to parse hook command: {}", hook))?;

        if args.is_empty() {
            anyhow::bail!("empty hook command");
        }

        let mut cmd = Command::new(&args[0]);
        cmd.args(&args[1..]).current_dir(manifest_dir);

        let output = cmd
            .output()
            .with_context(|| format!("failed to execute hook: {}", hook))?;

        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            anyhow::bail!("hook failed: {} (exit code: {})", hook, output.status);
        }
    }
    Ok(())
}
