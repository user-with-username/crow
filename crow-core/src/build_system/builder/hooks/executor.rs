use anyhow::Context;
use crow_utils::Logger;

pub fn execute_hooks(hooks: &[String], logger: Logger) -> anyhow::Result<()> {
    for hook in hooks {
        let cmd = shlex::split(hook).with_context(|| format!("Cannot parse hook: '{hook}'"))?;

        if cmd.is_empty() {
            continue;
        }

        let output = std::process::Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .with_context(|| format!("Failed to run: '{hook}'"))?;

        if !output.stdout.is_empty() {
            logger.log((), &String::from_utf8_lossy(&output.stdout), 0);
        }
        if !output.stderr.is_empty() {
            logger.log((), &String::from_utf8_lossy(&output.stderr), 0);
        }

        if !output.status.success() {
            anyhow::bail!("Hook failed: `{hook}`");
        }
    }
    Ok(())
}
