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
pub struct AsJsonArgs {}

pub struct AsJsonCommand {
    _args: AsJsonArgs,
}

impl AsJsonCommand {
    pub fn new(_args: AsJsonArgs) -> Self {
        Self { _args }
    }

    pub fn execute(self) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        // Load workspace and config
        let workspace = Workspace::load()?;
        let (config, _) =
            CrowConfig::find_in_tree(&current_dir)?;

        // Resolve toolchain
        let toolchain =
            toolchain::shared_toolchain(&config)?;

        let compiler_path =
            normalize_string_path(
                toolchain.compiler_path().to_string(),
            );

        let linker_path =
            if let Some(path) =
                config.build.linker.path()
            {
                normalize_string_path(
                    path.to_string(),
                )
            } else if toolchain
                .compiler_kind()
                .is_msvc()
            {
                normalize_string_path(
                    toolchain
                        .linker_path()
                        .to_string(),
                )
            } else {
                compiler_path.clone()
            };

        let archiver_path =
            if let Some(path) =
                config.build.archiver.path()
            {
                normalize_string_path(
                    path.to_string(),
                )
            } else {
                normalize_string_path(
                    toolchain
                        .archiver_path()
                        .to_string(),
                )
            };

        let is_workspace_member =
            workspace.members().any(
                |(_, path)| {
                    path == &current_dir
                        || current_dir
                            .starts_with(path)
                },
            );

        let info = ShowInfo {
            package: config.package.as_ref().map(
                |p| PackageInfo {
                    name: p.name.clone(),
                    version: p.version.clone(),

                    package_type:
                        package_type(
                            &p.r#type,
                        ),

                    description:
                        p.description.clone(),

                    authors: p
                        .authors
                        .clone()
                        .unwrap_or_default(),

                    license:
                        p.license.clone(),
                },
            ),

            workspace: WorkspaceInfo {
                root:
                    normalize_pathbuf(
                        workspace
                            .root
                            .clone(),
                    ),

                is_workspace_member,
            },

            toolchain: ToolchainInfo {
                compiler: ToolInfo {
                    kind: enum_lower(
                        toolchain
                            .compiler_kind(),
                    ),

                    path:
                        compiler_path,

                    flags: config
                        .build
                        .compiler
                        .flags()
                        .to_vec(),
                },

                linker: ToolInfo {
                    kind: enum_lower(
                        toolchain
                            .linker_kind(),
                    ),

                    path:
                        linker_path,

                    flags: vec![],
                },

                archiver: ToolInfo {
                    kind: enum_lower(
                        toolchain
                            .archiver_kind(),
                    ),

                    path:
                        archiver_path,

                    flags: vec![],
                },
            },

            build: BuildInfo {
                src_dirs: config
                    .build
                    .src_dirs
                    .iter()
                    .cloned()
                    .map(
                        normalize_pathbuf,
                    )
                    .collect(),

                include_dirs: config
                    .build
                    .include_dirs
                    .iter()
                    .cloned()
                    .map(
                        normalize_pathbuf,
                    )
                    .collect(),

                lib_dirs: config
                    .build
                    .lib_dirs
                    .iter()
                    .cloned()
                    .map(
                        normalize_pathbuf,
                    )
                    .collect(),

                libs:
                    config
                        .build
                        .libs
                        .clone(),

                test_dirs: config
                    .build
                    .test_dirs
                    .iter()
                    .cloned()
                    .map(
                        normalize_pathbuf,
                    )
                    .collect(),

                bench_dirs: config
                    .build
                    .bench_dirs
                    .iter()
                    .cloned()
                    .map(
                        normalize_pathbuf,
                    )
                    .collect(),

                parallelism:
                    config
                        .build
                        .parallelism,
            },

            paths: PathsInfo {
                target_dir:
                    normalize_pathbuf(
                        environment::Environment::target_dir(),
                    ),
            },
        };

        let json =
            serde_json::to_string_pretty(
                &info,
            )?;

        println!("{json}");

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
    description:
        Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    authors: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    license:
        Option<String>,
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

    include_dirs:
        Vec<PathBuf>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    lib_dirs:
        Vec<PathBuf>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    libs: Vec<String>,

    test_dirs:
        Vec<PathBuf>,

    bench_dirs:
        Vec<PathBuf>,

    parallelism: bool,
}

#[derive(Serialize)]
struct PathsInfo {
    target_dir: PathBuf,
}

fn package_type<T>(
    ty: &T,
) -> String
where
    T: std::fmt::Debug,
{
    format!("{:?}", ty)
        .split('(')
        .next()
        .unwrap_or("unknown")
        .to_lowercase()
}

fn enum_lower<T>(
    value: T,
) -> String
where
    T: std::fmt::Debug,
{
    format!("{:?}", value)
        .to_lowercase()
}

fn normalize_string_path(
    path: String,
) -> String {
    path.replace('\\', "/")
}

fn normalize_pathbuf(
    path: PathBuf,
) -> PathBuf {
    PathBuf::from(
        path.to_string_lossy()
            .replace('\\', "/"),
    )
}