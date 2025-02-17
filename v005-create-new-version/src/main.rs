use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use eyre::eyre;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing::warn;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env()?,
        )
        .init();

    info!("Ahoy, world!");

    // Find the root dir containing the versions
    let nanuak_dictionary_root_dir =
        get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir().await?;

    // Get the version objects
    let versions = get_versions(&nanuak_dictionary_root_dir).await?;

    // Print them
    for version in &versions {
        println!("{}", version.display());
    }

    // Get the version number to be used in the name of the next version
    let next_version_number = extract_next_version_number(&versions)
        .await
        .context(format!("Extracting next version number from {versions:?}"))?;

    // Prompt the user for the name of the next version, hinting the next version number
    let mut user_supplied_next_version_name = prompt_next_version_name(next_version_number).await?;

    // Repeat prompt until valid input received
    while let Err(e) = is_valid_version_name(&user_supplied_next_version_name) {
        warn!("Error: {}", e);
        user_supplied_next_version_name = prompt_next_version_name(next_version_number).await?;
    }
    let validated_next_version_name = user_supplied_next_version_name;

    // Identify the next version directory path
    let next_version_dir = nanuak_dictionary_root_dir.join(validated_next_version_name);

    // If the directory already exists, confirm y/n to proceed
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

    // Ask them if they would like to copy an existing version as a starter
    let copy_existing_version = dialoguer::Confirm::new()
        .with_prompt("Would you like to copy an existing version as a starter?")
        .interact()?;
    if copy_existing_version {
        // List the existing versions
        for (i, version) in versions.iter().enumerate() {
            println!("{}: {}", i, version.display());
        }
        // Prompt the user for the index of the version to copy
        let version_to_copy_index = dialoguer::Input::<usize>::new()
            .with_prompt("Enter the index of the version to copy")
            .interact()?;
        // Copy the selected version to the new version directory
        let version_to_copy = &versions[version_to_copy_index];
        tokio::fs::copy(version_to_copy, &next_version_dir).await?;
        info!(
            "Copied version {} to new version directory",
            version_to_copy.display()
        );
    } else {
        // Create the new version directory
        tokio::fs::create_dir(&next_version_dir).await?;
        info!(
            "Created new version directory: {}",
            next_version_dir.display()
        );
    }

    // Done
    Ok(())
}

fn is_valid_version_name(version_name: &str) -> eyre::Result<()> {
    let first_rule = version_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-');
    if !first_rule {
        bail!("Version name must be alphanumeric or hyphen");
    }
    let first_chunk = version_name.split('-').next().unwrap_or(version_name);
    // must begin with a v
    if !first_chunk.starts_with('v') {
        bail!("Version name must start with a 'v'");
    }
    // must have a number after the v
    if !first_chunk.chars().skip(1).all(|c| c.is_numeric()) {
        bail!("Version name must have a number after the 'v'");
    }
    Ok(())
}

async fn get_versions(nanuak_dictionary_root_dir: impl AsRef<Path>) -> eyre::Result<Vec<PathBuf>> {
    // Simply return the child directories sorted by name.
    let mut children = tokio::fs::read_dir(nanuak_dictionary_root_dir).await?;
    let mut versions = Vec::new();
    while let Some(entry) = children.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            if is_valid_version_name(
                &path
                    .file_name()
                    .ok_or_else(|| eyre!("No file name"))?
                    .to_string_lossy(),
            )
            .is_ok()
            {
                versions.push(path);
            }
        }
    }
    versions.sort();
    Ok(versions)
}

async fn is_valid_nanuak_dictionary_root_dir(nanuak_dictionary_root_dir: impl AsRef<Path>) -> bool {
    // Return true if the path is a directory with the exact name "Nanuak-Dictionary"
    let path = nanuak_dictionary_root_dir.as_ref();
    path.is_dir() && path.file_name() == Some(OsStr::new("Nanuak-Dictionary"))
}

async fn get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir() -> eyre::Result<PathBuf>
{
    let current_dir = tokio::fs::canonicalize(".").await?;
    if is_valid_nanuak_dictionary_root_dir(&current_dir).await {
        Ok(current_dir)
    } else {
        let parent_dir = current_dir
            .parent()
            .ok_or_else(|| eyre!("No parent directory"))?;
        if is_valid_nanuak_dictionary_root_dir(&parent_dir).await {
            Ok(parent_dir.to_path_buf())
        } else {
            Err(eyre!("No Nanuak-Dictionary directory found"))
        }
    }
}
async fn extract_next_version_number(versions: &[PathBuf]) -> eyre::Result<usize> {
    // Extract the version number from the last version directory.
    let last_version = versions.last().ok_or_else(|| eyre!("No versions found"))?;
    let last_version_number = extract_version_number(
        &last_version
            .file_name()
            .ok_or_else(|| eyre!("No file name on PathBuf last_version {last_version:?}"))?
            .to_string_lossy(),
    ).await?;
    Ok(last_version_number + 1)
}
async fn extract_version_number(version_name: &str) -> eyre::Result<usize> {
    let x = version_name
        .strip_prefix("v")
        .ok_or_eyre(format!("Missing 'v' prefix given {version_name:?}"))?;
    // if there is a hyphen, then strip everything after the first hyphen
    let x = x.split('-').next().unwrap_or(x);
    let x = x
        .parse::<usize>()
        .context(format!("Failed to parse version number from {x:?}"))?;
    Ok(x)
}

async fn prompt_next_version_name(next_version_number: usize) -> eyre::Result<String> {
    // Prompt the user for the next version name.
    let next_version_name = dialoguer::Input::<String>::new()
        .with_prompt(format!(
            "Enter the name for version v{}",
            next_version_number
        ))
        .interact()?;
    Ok(next_version_name)
}
