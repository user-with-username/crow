use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::json;
use url::Url;

pub struct GithubRepo {
    pub owner: String,
    pub repo: String,
}

impl GithubRepo {
    pub fn from_url(registry_url: &str) -> Result<Self> {
        let parsed = Url::parse(registry_url)
            .with_context(|| format!("Invalid registry URL: {registry_url}"))?;

        let segments: Vec<&str> = parsed.path().trim_matches('/').split('/').collect();
        anyhow::ensure!(
            segments.len() >= 2,
            "Invalid registry URL format: expected github.com/owner/repo, got {registry_url}"
        );

        Ok(Self {
            owner: segments[0].to_string(),
            repo: segments[1].trim_end_matches(".git").to_string(),
        })
    }

    pub fn create_pr(
        &self,
        token: &str,
        title: &str,
        head: &str,
        base: &str,
        body: &str,
        user_agent: &str,
    ) -> Result<()> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            self.owner, self.repo
        );

        let payload = json!({
            "title": title,
            "head":  head,
            "base":  base,
            "body":  body,
        });

        let response = Client::new()
            .post(&url)
            .header("Authorization", format!("token {token}"))
            .header("User-Agent", user_agent)
            .json(&payload)
            .send()
            .with_context(|| format!("Failed to send PR request to {url}"))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text()?;
            if status == 422 && text.contains("A pull request already exists") {
                return Ok(());
            }
            anyhow::bail!("GitHub API error ({status}): {text}");
        }

        Ok(())
    }
}
