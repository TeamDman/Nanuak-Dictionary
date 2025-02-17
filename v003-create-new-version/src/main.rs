use eyre::eyre;
use tracing::info;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;
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
    print_the_versions().await?;

    Ok(())
}

async fn get_versions(nanuak_dictionary_root_dir: impl AsRef<Path>) -> eyre::Result<Vec<PathBuf>> {
    // Simply return the child directories sorted by name.
    let mut children = tokio::fs::read_dir(nanuak_dictionary_root_dir).await?;
    let mut versions = Vec::new();
    while let Some(entry) = children.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            versions.push(path);
        }
    }
    versions.sort();
    Ok(versions)
}

async fn validate(nanuak_dictionary_root_dir: impl AsRef<Path>) -> bool {
    // Return true if the path is a directory with the exact name "Nanuak-Dictionary"
    let path = nanuak_dictionary_root_dir.as_ref();
    path.is_dir() && path.file_name() == Some(OsStr::new("Nanuak-Dictionary"))
}

async fn get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir() -> eyre::Result<PathBuf>
{
    let current_dir = tokio::fs::canonicalize(".").await?;
    if validate(&current_dir).await {
        Ok(current_dir)
    } else {
        let parent_dir = current_dir
            .parent()
            .ok_or_else(|| eyre!("No parent directory"))?;
        if validate(&parent_dir).await {
            Ok(parent_dir.to_path_buf())
        } else {
            Err(eyre!("No Nanuak-Dictionary directory found"))
        }
    }
}

async fn print_the_versions() -> eyre::Result<()> {
    let nanuak_dictionary_root_dir =
        get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir().await?;
    let versions = get_versions(&nanuak_dictionary_root_dir).await?;
    for version in versions {
        println!("{}", version.display());
    }
    Ok(())
}
