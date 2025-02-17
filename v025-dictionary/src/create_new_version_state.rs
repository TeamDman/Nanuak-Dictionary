use std::path::PathBuf;

use eyre::bail;
use eyre::Context;
use serde::Deserialize;
use serde::Serialize;
use tracing::info;
use v006_create_new_version::extract_next_version_number;
use v006_create_new_version::get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir;
use v006_create_new_version::get_versions;

use crate::state::State;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum CreateNewVersionState {
    #[default]
    DetermineWorkspaceCargoTomlPath,
    IdentifyNextVersionNumber {
        workspace_cargo_toml_path: PathBuf,
    },
    IdentifyNextVersionName {
        workspace_cargo_toml_path: PathBuf,
        next_version_number: usize,
    },
    IdentifyTemplateVersion {
        workspace_cargo_toml_path: PathBuf,
        next_version_number: usize,
        next_version_name: String,
        next_version_dir: PathBuf,
    },
    CreateNewVersion {
        workspace_cargo_toml_path: PathBuf,
        next_version_number: usize,
        next_version_name: String,
        next_version_dir: PathBuf,
        template_version_name: String,
        template_version_dir: PathBuf,
    },
    Done,
}
pub struct NewVersionBaseInformation {
    workspace_cargo_toml_path: PathBuf,
    next_version_number: usize,
    template_version_name: String,
    template_version_dir: PathBuf,
}
impl State for CreateNewVersionState {
    fn describe(&self) -> &'static str {
        match self {
            Self::DetermineWorkspaceCargoTomlPath => "Determine workspace Cargo.toml path",
            Self::IdentifyNextVersionNumber { .. } => "Identify next version number",
            Self::IdentifyNextVersionName { .. } => "Identify next version name",
            Self::IdentifyTemplateVersion { .. } => "Identify template version",
            Self::CreateNewVersion { .. } => "Create new version",
            Self::Done => "Done",
        }
    }

    async fn next(self) -> eyre::Result<Self>
    where
        Self: Sized,
    {
        match self {
            Self::DetermineWorkspaceCargoTomlPath => {
                info!("Find the root dir containing the versions");
                let nanuak_dictionary_root_dir =
                    get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir().await?;
                let workspace_cargo_toml_path = nanuak_dictionary_root_dir.join("Cargo.toml");
                if tokio::fs::try_exists(&workspace_cargo_toml_path).await? == false {
                    bail!(
                        "Cargo.toml not found at {}",
                        workspace_cargo_toml_path.display()
                    );
                }

                Ok(Self::IdentifyNextVersionNumber {
                    workspace_cargo_toml_path,
                })
            }
            Self::IdentifyNextVersionNumber {
                workspace_cargo_toml_path,
            } => {
                let versions = get_versiocns(&nanuak_dictionary_root_dir).await?;
                for version in &versions {
                    println!("{}", version.display());
                }
                let next_version_number = extract_next_version_number(&versions)
                    .await
                    .context(format!("Extracting next version number from {versions:?}"))?;

                Ok(Self::IdentifyNextVersionName {
                    workspace_cargo_toml_path,
                    next_version_number,
                })
            }
            Self::IdentifyNextVersionName {
                workspace_cargo_toml_path,
                next_version_number,
            } => {
                let next_version_name = todo!("identify next version name");
                Ok(Self::IdentifyTemplateVersion {
                    workspace_cargo_toml_path,
                    next_version_number,
                    next_version_name,
                    next_version_dir: todo!("identify next version directory"),
                })
            }
            Self::IdentifyTemplateVersion {
                workspace_cargo_toml_path,
                next_version_number,
                next_version_name,
                next_version_dir,
            } => {
                let template_version_name = todo!("identify template version name");
                let template_version_dir = todo!("identify template version directory");
                Ok(Self::CreateNewVersion {
                    workspace_cargo_toml_path,
                    next_version_number,
                    next_version_name,
                    next_version_dir,
                    template_version_name,
                    template_version_dir,
                })
            }
            Self::CreateNewVersion {
                workspace_cargo_toml_path,
                next_version_number,
                next_version_name,
                next_version_dir,
                template_version_name,
                template_version_dir,
            } => {
                todo!("create new version");
                Ok(Self::Done)
            }

            Self::Done => Ok(Self::Done),
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}
