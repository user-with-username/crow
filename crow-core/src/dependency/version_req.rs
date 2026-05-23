use anyhowed::Result;
use semver::{Version, VersionReq as SemverVersionReq};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionReq {
    Semver(SemverVersionReq),
    Wildcard(Vec<u32>),
    Any,
    Exact(Version),
}

impl VersionReq {
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();

        if trimmed == "*" {
            return Ok(VersionReq::Any);
        }

        if trimmed.contains('*') {
            return Self::parse_wildcard(trimmed);
        }

        match SemverVersionReq::parse(trimmed) {
            Ok(req) => {
                if req.comparators.len() == 1 && req.comparators[0].op == semver::Op::Exact {
                    if let Ok(version) = Version::parse(trimmed) {
                        return Ok(VersionReq::Exact(version));
                    }
                }
                Ok(VersionReq::Semver(req))
            }
            Err(_) => {
                if let Ok(version) = Version::parse(trimmed) {
                    Ok(VersionReq::Exact(version))
                } else {
                    anyhowed::bail!("invalid version requirement: {}", trimmed)
                }
            }
        }
    }

    fn parse_wildcard(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split('.').collect();
        let mut prefix = Vec::new();

        for (i, part) in parts.iter().enumerate() {
            if *part == "*" {
                for remaining in &parts[i + 1..] {
                    if *remaining != "*" {
                        anyhowed::bail!("invalid wildcard pattern: '*' must be at the end");
                    }
                }
                break;
            }

            let num: u32 = part
                .parse()
                .map_err(|_| anyhowed::anyhow!("invalid version number: {}", part))?;
            prefix.push(num);
        }

        Ok(VersionReq::Wildcard(prefix))
    }

    pub fn matches(&self, version_str: &str) -> bool {
        let version = match Version::parse(version_str) {
            Ok(v) => v,
            Err(_) => return false,
        };

        match self {
            VersionReq::Semver(req) => req.matches(&version),
            VersionReq::Wildcard(prefix) => {
                let parts: Vec<u32> = version
                    .to_string()
                    .split('.')
                    .filter_map(|s| s.parse().ok())
                    .collect();

                parts.len() >= prefix.len() && &parts[..prefix.len()] == prefix
            }
            VersionReq::Any => true,
            VersionReq::Exact(exact) => &version == exact,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            VersionReq::Semver(req) => req.to_string(),
            VersionReq::Wildcard(prefix) => {
                if prefix.is_empty() {
                    "*".to_string()
                } else {
                    format!(
                        "{}.*",
                        prefix
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(".")
                    )
                }
            }
            VersionReq::Any => "*".to_string(),
            VersionReq::Exact(version) => version.to_string(),
        }
    }
}

impl fmt::Display for VersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for VersionReq {
    type Err = anyhowed::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
