use std::path::PathBuf;

use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use tracing::info;
use tracing::warn;
use v006_create_new_version::extract_next_version_number;
use v006_create_new_version::get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir;
use v006_create_new_version::get_versions;
use v006_create_new_version::is_valid_version_name;
use v006_create_new_version::prompt_next_version_name;

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
        workspace_dir: PathBuf,
    },
    IdentifyTemplateVersion {
        workspace_dir: PathBuf,
        workspace_cargo_toml_path: PathBuf,
        next_version_number: usize,
        next_version_name: String,
        next_version_dir: PathBuf,
    },
    CreateNewVersionFromTemplate {
        workspace_cargo_toml_path: PathBuf,
        next_version_number: usize,
        next_version_name: String,
        next_version_dir: PathBuf,
        template_version_name: String,
        template_version_dir: PathBuf,
    },
    ApplyFileChanges {
        workspace_cargo_toml_path: PathBuf,
        next_version_number: usize,
        next_version_name: String,
        next_version_dir: PathBuf,
    },
    Done,
}
#[async_trait::async_trait]
impl State for CreateNewVersionState {
    fn describe(&self) -> &'static str {
        match self {
            Self::DetermineWorkspaceCargoTomlPath => "Determine workspace Cargo.toml path",
            Self::IdentifyNextVersionNumber { .. } => "Identify next version number",
            Self::IdentifyNextVersionName { .. } => "Identify next version name",
            Self::IdentifyTemplateVersion { .. } => "Identify template version",
            Self::CreateNewVersionFromTemplate { .. } => "Create new version from template",
            Self::ApplyFileChanges { .. } => "Apply file changes",
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
                let workspace_dir = workspace_cargo_toml_path
                    .parent()
                    .ok_or_eyre("No parent dir")?
                    .to_path_buf();
                let versions = get_versions(&workspace_dir).await?;
                for version in &versions {
                    println!("{}", version.display());
                }
                let next_version_number = extract_next_version_number(&versions)
                    .await
                    .context(format!("Extracting next version number from {versions:?}"))?;

                Ok(Self::IdentifyNextVersionName {
                    workspace_dir,
                    workspace_cargo_toml_path,
                    next_version_number,
                })
            }
            Self::IdentifyNextVersionName {
                workspace_dir,
                workspace_cargo_toml_path,
                next_version_number,
            } => {
                info!(
                    "Prompt the user for the name of the next version, hinting the next version number"
                );
                let mut user_supplied_next_version_name =
                    prompt_next_version_name(next_version_number).await?;

                info!("Repeat prompt until valid input received");
                while let Err(e) = is_valid_version_name(&user_supplied_next_version_name) {
                    warn!("Error: {}", e);
                    user_supplied_next_version_name =
                        prompt_next_version_name(next_version_number).await?;
                }
                let validated_next_version_name = user_supplied_next_version_name;

                info!("Identify the next version directory path");
                let next_version_dir = workspace_dir.join(&validated_next_version_name);

                info!("If the directory already exists, confirm y/n to proceed");
                if next_version_dir.exists() {
                    let proceed = dialoguer::Confirm::new()
                        .with_prompt(format!(
                            "Directory {} already exists. Proceed?",
                            next_version_dir.display()
                        ))
                        .interact()?;
                    if !proceed {
                        bail!("User chose not to proceed");
                    }
                }
                Ok(Self::IdentifyTemplateVersion {
                    workspace_dir,
                    workspace_cargo_toml_path,
                    next_version_number,
                    next_version_name: validated_next_version_name,
                    next_version_dir,
                })
            }
            Self::IdentifyTemplateVersion {
                workspace_dir,
                workspace_cargo_toml_path,
                next_version_number,
                next_version_name,
                next_version_dir,
            } => {
                let versions = get_versions(&workspace_dir).await?;
                let chosen = pick(FzfArgs {
                    choices: versions
                        .into_iter()
                        .map(|version| Choice {
                            key: version.display().to_string(),
                            value: version,
                        })
                        .collect_vec(),
                    header: Some("Choose a version to copy".to_string()),
                    prompt: None,
                })?;
                Ok(Self::CreateNewVersionFromTemplate {
                    workspace_cargo_toml_path,
                    next_version_number,
                    next_version_name,
                    next_version_dir,
                    template_version_name: chosen
                        .value
                        .file_name()
                        .ok_or_eyre(format!("No file name for {:?}", chosen.value))?
                        .to_string_lossy()
                        .to_string(),
                    template_version_dir: chosen.value,
                })
            }
            Self::CreateNewVersionFromTemplate {
                workspace_cargo_toml_path,
                next_version_number,
                next_version_name,
                next_version_dir,
                template_version_name: _,
                template_version_dir,
            } => {
                v006_create_new_version::copy_dir_all(&template_version_dir, &next_version_dir)
                    .await
                    .context("Copying the reference version to the new version")?;
                info!(
                    "Copied version {} to new version directory",
                    template_version_dir.display()
                );

                Ok(Self::ApplyFileChanges {
                    workspace_cargo_toml_path,
                    next_version_number,
                    next_version_name,
                    next_version_dir,
                })
            }
            Self::ApplyFileChanges {
                workspace_cargo_toml_path: _,
                next_version_number: _,
                next_version_name,
                next_version_dir,
            } => {
                info!("Applying the new version name to the Cargo.toml");
                v007_create_new_version::apply_file_changes_for_new_version_name(
                    next_version_dir,
                    &next_version_name,
                )
                .await
                .context("Applying the new version name to the Cargo.toml")?;
                Ok(Self::Done)
            }

            Self::Done => Ok(Self::Done),
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}
