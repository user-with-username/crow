use crate::config::CrowConfig;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub root: PathBuf,
    pub config: CrowConfig,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub is_wheel: bool,
    pub build_flags: Vec<String>,
    /// Features requested from a registry dependency (e.g. ["asio"] for boost).
    /// Empty for system deps and registry deps without features.
    pub features: Vec<String>,
    /// Auto-generated link libs from features (e.g. ["boost_asio"]).
    /// Only populated for registry deps that have features and no explicit libs.
    pub auto_libs: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedDependencyBuild {
    pub packages: Vec<ResolvedPackage>,
    pub include_dirs: Vec<PathBuf>,
    pub libs: Vec<String>,
    pub lib_paths: Vec<PathBuf>,
    pub max_standard: Option<String>,
    pub system_libs: Vec<String>,
    /// Maps system dep name → enabled features (e.g. "boost" → ["system", "filesystem"])
    pub system_features: HashMap<String, Vec<String>>,
    /// Maps system dep name → libs (e.g. "boost" → ["boost_system", "boost_filesystem"])
    pub system_libs_map: HashMap<String, Vec<String>>,
    /// Maps registry dep name → enabled features (e.g. "boost" → ["asio"])
    pub registry_features: HashMap<String, Vec<String>>,
}

impl ResolvedDependencyBuild {
    pub fn absorb_wheel_artifacts(&mut self, artifacts: &crate::dependency::WheelArtifacts) {
        self.include_dirs
            .extend(artifacts.include_dirs.iter().cloned());
        self.lib_paths.extend(artifacts.lib_paths.iter().cloned());
        self.libs.extend(artifacts.lib_names.iter().cloned());

        self.include_dirs.sort_unstable();
        self.include_dirs.dedup();

        self.lib_paths.sort_unstable();
        self.lib_paths.dedup();

        self.libs.sort();
        self.libs.dedup();
    }
}
