use anyhowed::{bail, Context, Result};
use clap::Args;
use crow_utils::status;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const REPO: &str = "user-with-username/crow";

#[derive(Args)]
pub struct SelfUpdateArgs {}

pub struct SelfUpdateCommand {
    _args: SelfUpdateArgs,
}

impl SelfUpdateCommand {
    pub fn new(args: SelfUpdateArgs) -> Self {
        Self { _args: args }
    }

    pub fn execute(self) -> Result<()> {
        let current_exe =
            std::env::current_exe().context("failed to determine current executable path")?;

        let artifact = Self::detect_artifact();
        if artifact == "unknown" {
            bail!(
                "unsupported platform: {}-{}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
        }
        status!("Fetching", "latest release metadata...");

        let client = reqwest::blocking::Client::builder()
            .user_agent("crow-self-updater/1.0")
            .build()
            .context("failed to build HTTP client")?;

        let release: serde_json::Value = client
            .get(format!(
                "https://api.github.com/repos/{REPO}/releases/latest"
            ))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .context("failed to fetch release metadata")?
            .error_for_status()
            .context("release metadata request failed")?
            .json()
            .context("failed to parse release metadata")?;

        let tag = release["tag_name"]
            .as_str()
            .context("missing tag_name in release")?;
        status!("Latest", "{}", tag);

        let expected_hash = Self::get_hash_from_assets(&release, artifact)?;

        let current_data = fs::read(&current_exe).context("failed to read current executable")?;
        let current_hash = format!("{:x}", Sha256::digest(&current_data));

        if current_hash == expected_hash {
            status!("Up to date", "already running the latest version ({tag})");
            return Ok(());
        }

        let download_url = Self::get_download_url_from_assets(&release, artifact)?;

        let data = client
            .get(&download_url)
            .send()
            .context("failed to download release artifact")?
            .error_for_status()
            .context("download request failed")?
            .bytes()
            .context("failed to read download response")?;

        let downloaded_hash = format!("{:x}", Sha256::digest(&data));
        if downloaded_hash != expected_hash {
            bail!("hash mismatch: expected {expected_hash}, got {downloaded_hash}");
        }

        Self::replace_binary(&current_exe, &data)?;
        status!("Updated", "crow has been updated to {tag}");

        Ok(())
    }

    fn detect_artifact() -> &'static str {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => "linux-x86_64",
            ("linux", "aarch64") => "linux-arm64",
            ("macos", "x86_64") => "macos-x86_64",
            ("macos", "aarch64") => "macos-arm64",
            ("windows", "x86_64") => "windows-x64.exe",
            _ => "unknown",
        }
    }

    fn get_hash_from_assets(release: &serde_json::Value, artifact: &str) -> Result<String> {
        let assets = release["assets"]
            .as_array()
            .context("no assets array in release")?;

        let asset = assets
            .iter()
            .find(|a| a["name"].as_str() == Some(artifact))
            .context(format!("asset '{}' not found in release", artifact))?;

        let digest = asset["digest"]
            .as_str()
            .context("asset missing digest field")?;

        let hash = digest
            .strip_prefix("sha256:")
            .context("invalid digest format, expected 'sha256:hash'")?
            .trim()
            .to_lowercase();

        Ok(hash)
    }

    fn get_download_url_from_assets(release: &serde_json::Value, artifact: &str) -> Result<String> {
        let assets = release["assets"]
            .as_array()
            .context("no assets array in release")?;

        let asset = assets
            .iter()
            .find(|a| a["name"].as_str() == Some(artifact))
            .context(format!("asset '{}' not found in release", artifact))?;

        let url = asset["browser_download_url"]
            .as_str()
            .context("asset missing browser_download_url")?
            .to_string();

        Ok(url)
    }

    fn replace_binary(exe: &Path, data: &[u8]) -> Result<()> {
        let tmp = exe.with_extension("tmp");
        fs::write(&tmp, data).context("failed to write new binary to temp file")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&tmp)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&tmp, perms)?;
            fs::rename(&tmp, exe).context("failed to replace binary")?;
            Ok(())
        }

        #[cfg(windows)]
        {
            use crow_utils::find_executable;
            use std::process::Command;

            let tmp_str = tmp.to_str().context("invalid tmp path")?;
            let exe_str = exe.to_str().context("invalid exe path")?;

            let ps_command = format!(
                "Start-Sleep -Seconds 2; \
                 Move-Item -Force -Path '{}' -Destination '{}'; \
                 Remove-Item -LiteralPath $MyInvocation.MyCommand.Path -Force",
                tmp_str, exe_str
            );

            let ps_script = exe.with_extension("ps1");
            fs::write(&ps_script, &ps_command)?;

            let powershell = find_executable("powershell")?;

            Command::new(&powershell)
                .args([
                    "-WindowStyle",
                    "Hidden",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    ps_script.to_str().unwrap(),
                ])
                .spawn()
                .context("failed to spawn PowerShell replacement script")?;

            Ok(())
        }
    }
}
