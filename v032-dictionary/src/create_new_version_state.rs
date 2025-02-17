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
use v007_create_new_version::cargo_toml;

use crate::state::State;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum CreateNewVersionState {
    #[default]
    DetermineWorkspaceCargoTomlPath,
    IdentifyNextVersionNumber {
        workspace_dir: PathBuf,
    },
    IdentifyNextVersionName {
        workspace_dir: PathBuf,
        next_version_number: usize,
    },
    IdentifyTemplateVersion {
        workspace_dir: PathBuf,
        next_version_name: String,
        next_version_dir: PathBuf,
    },
    CreateNewVersionFromTemplate {
        workspace_dir: PathBuf,
        next_version_name: String,
        next_version_dir: PathBuf,
        template_version_name: String,
        template_version_dir: PathBuf,
    },
    UpdateWorkspaceCargoToml {
        workspace_dir: PathBuf,
        next_version_name: String,
        next_version_dir: PathBuf,
        template_version_name: String,
        template_version_dir: PathBuf,
    },
    UpdateVersionCargoToml {
        workspace_dir: PathBuf,
        next_version_name: String,
        next_version_dir: PathBuf,
        template_version_name: String,
        template_version_dir: PathBuf,
    },
    UpdateMain {
        workspace_dir: PathBuf,
        next_version_name: String,
        next_version_dir: PathBuf,
        template_version_name: String,
        template_version_dir: PathBuf,
    },
    Done,
}
#[async_trait::async_trait]
impl State for CreateNewVersionState {
    fn describe(&self) -> String {
        match self {
            Self::DetermineWorkspaceCargoTomlPath => "Determine workspace Cargo.toml path",
            Self::IdentifyNextVersionNumber { .. } => "Identify next version number",
            Self::IdentifyNextVersionName { .. } => "Identify next version name",
            Self::IdentifyTemplateVersion { .. } => "Identify template version",
            Self::CreateNewVersionFromTemplate { .. } => "Create new version from template",
            Self::UpdateWorkspaceCargoToml { .. } => "Update workspace Cargo.toml",
            Self::UpdateVersionCargoToml { .. } => "Update version Cargo.toml",
            Self::UpdateMain { .. } => "Update main",
            Self::Done => "Done",
        }
        .to_string()
    }

    async fn next(self) -> eyre::Result<Self>
    where
        Self: Sized,
    {
        match self {
            Self::DetermineWorkspaceCargoTomlPath => {
                info!("Find the root dir containing the versions");
                let workspace_dir =
                    get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir().await?;

                Ok(Self::IdentifyNextVersionNumber { workspace_dir })
            }
            Self::IdentifyNextVersionNumber { workspace_dir } => {
                let versions = get_versions(&workspace_dir).await?;
                for version in &versions {
                    println!("{}", version.display());
                }
                let next_version_number = extract_next_version_number(&versions)
                    .await
                    .context(format!("Extracting next version number from {versions:?}"))?;

                Ok(Self::IdentifyNextVersionName {
                    workspace_dir,
                    next_version_number,
                })
            }
            Self::IdentifyNextVersionName {
                workspace_dir,
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
                    next_version_name: validated_next_version_name,
                    next_version_dir,
                })
            }
            Self::IdentifyTemplateVersion {
                workspace_dir,
                next_version_name,
                next_version_dir,
            } => {
                let versions = get_versions(&workspace_dir).await?;
                let mut choices = versions
                    .into_iter()
                    .map(|version| Choice {
                        key: version.display().to_string(),
                        value: version,
                    })
                    .collect_vec();
                choices.reverse();
                let chosen = pick(FzfArgs {
                    choices,
                    header: Some("Choose a version to copy".to_string()),
                    prompt: None,
                })?;
                Ok(Self::CreateNewVersionFromTemplate {
                    workspace_dir,
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
                workspace_dir,
                next_version_name,
                next_version_dir,
                template_version_name,
                template_version_dir,
            } => {
                v006_create_new_version::copy_dir_all(&template_version_dir, &next_version_dir)
                    .await
                    .context("Copying the reference version to the new version")?;
                info!(
                    "Copied version {} to new version directory",
                    template_version_dir.display()
                );

                Ok(Self::UpdateWorkspaceCargoToml {
                    workspace_dir,
                    next_version_name,
                    next_version_dir,
                    template_version_name,
                    template_version_dir,
                })
            }
            Self::UpdateWorkspaceCargoToml {
                workspace_dir,
                next_version_name,
                next_version_dir,
                template_version_name,
                template_version_dir,
            } => {
                let workspace_cargo_toml_path = workspace_dir.join("Cargo.toml");
                if tokio::fs::try_exists(&workspace_cargo_toml_path).await? == false {
                    bail!(
                        "Cargo.toml not found at {}",
                        workspace_cargo_toml_path.display()
                    );
                }
                info!(
                    "Add the new version name to {}",
                    workspace_cargo_toml_path.display()
                );
                let workspace_cargo_toml =
                    tokio::fs::read_to_string(&workspace_cargo_toml_path).await?;
                let mut workspace_cargo_toml: cargo_toml::CargoToml =
                    toml::from_str(&workspace_cargo_toml).context(format!(
                        "Interpreting cargo toml from {}",
                        workspace_cargo_toml_path.display()
                    ))?;
                workspace_cargo_toml
                    .workspace
                    .as_mut()
                    .ok_or_eyre("No workspace")?
                    .members
                    .push(next_version_name.clone());
                let None = workspace_cargo_toml
                    .workspace
                    .as_mut()
                    .ok_or_eyre("No workspace")?
                    .dependencies
                    .0
                    .insert(
                        next_version_name.clone(),
                        cargo_toml::Dependency::Path {
                            path: next_version_name.to_string(),
                            features: None,
                        },
                    )
                else {
                    bail!(
                        "Dependency already exists trying to insert {:?} to {} which exists as {:#?}",
                        next_version_name,
                        workspace_cargo_toml_path.display(),
                        workspace_cargo_toml
                    );
                };
                tokio::fs::write(
                    &workspace_cargo_toml_path,
                    toml::to_string(&workspace_cargo_toml)?,
                )
                .await?;
                Ok(Self::UpdateVersionCargoToml {
                    workspace_dir,
                    next_version_name,
                    next_version_dir,
                    template_version_name,
                    template_version_dir,
                })
            }
            Self::UpdateVersionCargoToml {
                workspace_dir,
                next_version_name,
                next_version_dir,
                template_version_name,
                template_version_dir,
            } => {
                info!(
                    "replace the old version name in {}/Cargo.toml with the new version name",
                    next_version_name
                );
                let cargo_toml_path = next_version_dir.join("Cargo.toml");
                let cargo_toml =
                    tokio::fs::read_to_string(&cargo_toml_path)
                        .await
                        .context(format!(
                            "Reading {} as CargoToml",
                            cargo_toml_path.display()
                        ))?;
                let mut cargo_toml: cargo_toml::CargoToml = toml::from_str(&cargo_toml).context(
                    format!("Parsing {} as CargoToml", cargo_toml_path.display()),
                )?;
                cargo_toml
                    .package
                    .as_mut()
                    .ok_or_eyre(format!(
                        "Expected \"package\" to be present in {}",
                        cargo_toml_path.display()
                    ))?
                    .name = next_version_name.to_string();
                tokio::fs::write(&cargo_toml_path, toml::to_string(&cargo_toml)?).await?;

                Ok(Self::UpdateMain {
                    workspace_dir,
                    next_version_name,
                    next_version_dir,
                    template_version_name,
                    template_version_dir,
                })
            }
            Self::UpdateMain {
                workspace_dir: _,
                next_version_name,
                next_version_dir,
                template_version_name,
                template_version_dir: _,
            } => {
                let main_rs_path = next_version_dir.join("src").join("main.rs");
                info!("Update the main.rs file at {}", main_rs_path.display());
                let main_rs = tokio::fs::read_to_string(&main_rs_path).await?;
                let main_rs = main_rs.replace(&template_version_name, &next_version_name);
                tokio::fs::write(&main_rs_path, main_rs).await?;
                Ok(Self::Done)
            }
            Self::Done => Ok(Self::Done),
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}
