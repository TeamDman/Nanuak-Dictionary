use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use std::path::PathBuf;
use tracing::info;
use tracing::warn;
use v006_create_new_version::copy_dir_all;
use v006_create_new_version::extract_next_version_number;
use v006_create_new_version::get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir;
use v006_create_new_version::get_versions;
use v006_create_new_version::is_valid_version_name;
use v006_create_new_version::prompt_next_version_name;

pub async fn create_new_version() -> eyre::Result<()> {
    info!("Find the root dir containing the versions");
    let nanuak_dictionary_root_dir =
        get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir().await?;

    info!("Get the version objects");
    let versions = get_versions(&nanuak_dictionary_root_dir).await?;

    info!("Print them");
    for version in &versions {
        println!("{}", version.display());
    }

    info!("Get the version number to be used in the name of the next version");
    let next_version_number = extract_next_version_number(&versions)
        .await
        .context(format!("Extracting next version number from {versions:?}"))?;

    info!("Checking if the user wants to accept all suggested defaults");
    let accept_all_defaults = dialoguer::Confirm::new()
        .with_prompt("Accept all defaults?")
        .interact()?;
    if accept_all_defaults {
        info!("User accepted all defaults");
        let next_version_name = format!("v{:03}", next_version_number);
        let next_version_dir = nanuak_dictionary_root_dir.join(&next_version_name);
        if next_version_dir.exists() {
            bail!(
                "Directory {} already exists. Cannot proceed with all defaults",
                next_version_dir.display()
            );
        }
        tokio::fs::create_dir(&next_version_dir).await?;
        info!(
            "Created new version directory: {}",
            next_version_dir.display()
        );
        apply_file_changes_for_new_version_name(next_version_dir, &next_version_name)
            .await
            .context("Applying the new version name to the Cargo.toml")?;
        return Ok(());
    }

    info!("Prompt the user for the name of the next version, hinting the next version number");
    let mut user_supplied_next_version_name = prompt_next_version_name(next_version_number).await?;

    info!("Repeat prompt until valid input received");
    while let Err(e) = is_valid_version_name(&user_supplied_next_version_name) {
        warn!("Error: {}", e);
        user_supplied_next_version_name = prompt_next_version_name(next_version_number).await?;
    }
    let validated_next_version_name = user_supplied_next_version_name;

    info!("Identify the next version directory path");
    let next_version_dir = nanuak_dictionary_root_dir.join(&validated_next_version_name);

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

    info!("Ask them if they would like to copy an existing version as a starter");
    let copy_existing_version = dialoguer::Confirm::new()
        .with_prompt("Would you like to copy an existing version as a starter?")
        .interact()?;
    if copy_existing_version {
        info!("List the existing versions");
        for (i, version) in versions.iter().enumerate() {
            println!("{}: {}", i, version.display());
        }
        info!("Prompt the user for the index of the version to copy");
        let version_to_copy_index = dialoguer::Input::<usize>::new()
            .with_prompt("Enter the index of the version to copy")
            .interact()?;
        info!("Copy the selected version to the new version directory");
        let version_to_copy = &versions[version_to_copy_index];
        copy_dir_all(version_to_copy, &next_version_dir)
            .await
            .context("Copying the reference version to the new version")?;
        info!(
            "Copied version {} to new version directory",
            version_to_copy.display()
        );

        info!("Applying the new version name to the Cargo.toml");
        apply_file_changes_for_new_version_name(next_version_dir, &validated_next_version_name)
            .await
            .context("Applying the new version name to the Cargo.toml")?;
    } else {
        info!("Create the new version directory");
        tokio::fs::create_dir(&next_version_dir).await?;
        info!(
            "Created new version directory: {}",
            next_version_dir.display()
        );
    }

    info!("Done");
    Ok(())
}

pub async fn apply_file_changes_for_new_version_name(
    new_version_dir: PathBuf,
    new_version_name: &str,
) -> eyre::Result<()> {
    info!(
        "replace the old version name in {}/Cargo.toml with the new version name",
        new_version_name
    );
    let cargo_toml_path = new_version_dir.join("Cargo.toml");
    let cargo_toml = tokio::fs::read_to_string(&cargo_toml_path).await?;
    let mut cargo_toml: cargo_toml::CargoToml = toml::from_str(&cargo_toml).context(format!(
        "Parsing {} as CargoToml",
        cargo_toml_path.display()
    ))?;
    cargo_toml
        .package
        .as_mut()
        .ok_or_eyre(format!(
            "Expected \"package\" to be present in {}",
            cargo_toml_path.display()
        ))?
        .name = new_version_name.to_string();
    tokio::fs::write(&cargo_toml_path, toml::to_string(&cargo_toml)?).await?;

    let workspace_cargo_toml_path = new_version_dir
        .parent()
        .ok_or_eyre("Could not get parent of new version directory")?
        .join("Cargo.toml");
    info!(
        "Add the new version name to {}",
        workspace_cargo_toml_path.display()
    );
    let workspace_cargo_toml = tokio::fs::read_to_string(&workspace_cargo_toml_path).await?;
    let mut workspace_cargo_toml: cargo_toml::CargoToml = toml::from_str(&workspace_cargo_toml)
        .context(format!(
            "Interpreting cargo toml from {}",
            workspace_cargo_toml_path.display()
        ))?;
    let new_version_name = new_version_name.to_string();
    workspace_cargo_toml
        .workspace
        .as_mut()
        .ok_or_eyre("No workspace")?
        .members
        .push(new_version_name.clone());
    let None = workspace_cargo_toml
        .workspace
        .as_mut()
        .ok_or_eyre("No workspace")?
        .dependencies
        .0
        .insert(
            new_version_name.clone(),
            cargo_toml::Dependency::Path {
                path: new_version_name.to_string(),
                features: None,
            },
        )
    else {
        bail!(
            "Dependency already exists trying to insert {:?} to {} which exists as {:#?}",
            new_version_name,
            workspace_cargo_toml_path.display(),
            workspace_cargo_toml
        );
    };
    tokio::fs::write(
        &workspace_cargo_toml_path,
        toml::to_string(&workspace_cargo_toml)?,
    )
    .await?;

    Ok(())
}

pub mod cargo_toml {
    use std::collections::HashMap;

    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct CargoToml {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub package: Option<Package>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dependencies: Option<Dependencies>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dev_dependencies: Option<Dependencies>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub workspace: Option<Workspace>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Package {
        pub name: String,
        pub version: String,
        pub edition: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Workspace {
        pub resolver: String,
        pub members: Vec<String>,
        pub dependencies: Dependencies,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Dependencies(pub HashMap<String, Dependency>);

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    pub enum Dependency {
        Path {
            path: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            features: Option<Vec<String>>,
        },
        RawVersion(String),
        Version {
            version: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            features: Option<Vec<String>>,
        },
    }
}
