use anyhowed::Result;
use clap::Args;
use crow_core::{
    builder::toolchain,
    config::{CrowConfig, Workspace},
};
use crow_utils::environment;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Args)]
pub struct MetadataArgs {}

pub struct MetadataCommand {
    _args: MetadataArgs,
}

impl MetadataCommand {
    pub fn new(_args: MetadataArgs) -> Self {
        Self { _args }
    }

    pub fn execute(self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let workspace = Workspace::load()?;
        let (config, _) = CrowConfig::find_in_tree(&current_dir)?;
        let toolchain = toolchain::shared_toolchain(&config)?;

        let compiler_path = toolchain.compiler_path().to_string();
        let linker_path = config.build.linker
            .path()
            .map(|p| p.to_string())
            .or_else(|| toolchain.compiler_kind().is_msvc().then(|| toolchain.linker_path().to_string()))
            .unwrap_or_else(|| compiler_path.clone());
        let archiver_path = config.build.archiver
            .path()
            .map(|p| p.to_string())
            .unwrap_or_else(|| toolchain.archiver_path().to_string());

        let workspace_root = workspace.root.clone();
        let is_workspace_member = workspace.members().any(|(_, path)| path == &current_dir || current_dir.starts_with(path));

        let info = ShowInfo {
            package: config.package.as_ref().map(|p| PackageInfo {
                name: p.name.clone(),
                version: p.version.clone(),
                package_type: format!("{:?}", p.r#type).split('(').next().unwrap_or("unknown").to_lowercase(),
                description: p.description.clone(),
                authors: p.authors.clone().unwrap_or_default(),
                license: p.license.clone(),
            }),
            workspace: WorkspaceInfo {
                root: workspace_root,
                is_workspace_member,
            },
            toolchain: ToolchainInfo {
                compiler: ToolInfo {
                    kind: format!("{:?}", toolchain.compiler_kind()).to_lowercase(),
                    path: compiler_path,
                    flags: config.build.compiler.flags().to_vec(),
                },
                linker: ToolInfo {
                    kind: format!("{:?}", toolchain.linker_kind()).to_lowercase(),
                    path: linker_path,
                    flags: vec![],
                },
                archiver: ToolInfo {
                    kind: format!("{:?}", toolchain.archiver_kind()).to_lowercase(),
                    path: archiver_path,
                    flags: vec![],
                },
            },
            build: BuildInfo {
                src_dirs: config.build.src_dirs,
                include_dirs: config.build.include_dirs,
                lib_dirs: config.build.lib_dirs,
                libs: config.build.libs,
                test_dirs: config.build.test_dirs,
                bench_dirs: config.build.bench_dirs,
                parallelism: config.build.parallelism,
            },
            paths: PathsInfo {
                target_dir: environment::Environment::target_dir(),
            },
        };

        println!("{}", serde_json::to_string_pretty(&info)?);
        Ok(())
    }
}

#[derive(Serialize)]
struct ShowInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    package: Option<PackageInfo>,
    workspace: WorkspaceInfo,
    toolchain: ToolchainInfo,
    build: BuildInfo,
    paths: PathsInfo,
}

#[derive(Serialize)]
struct PackageInfo {
    name: String,
    version: String,
    #[serde(rename = "type")]
    package_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
}

#[derive(Serialize)]
struct WorkspaceInfo {
    root: PathBuf,
    is_workspace_member: bool,
}

#[derive(Serialize)]
struct ToolchainInfo {
    compiler: ToolInfo,
    linker: ToolInfo,
    archiver: ToolInfo,
}

#[derive(Serialize)]
struct ToolInfo {
    kind: String,
    path: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    flags: Vec<String>,
}

#[derive(Serialize)]
struct BuildInfo {
    src_dirs: Vec<PathBuf>,
    include_dirs: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    lib_dirs: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    libs: Vec<String>,
    test_dirs: Vec<PathBuf>,
    bench_dirs: Vec<PathBuf>,
    parallelism: bool,
}

#[derive(Serialize)]
struct PathsInfo {
    target_dir: PathBuf,
}