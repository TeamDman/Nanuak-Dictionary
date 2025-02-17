use tracing::{info, level_filters::LevelFilter};
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

    print_the_versions()?;
    
    Ok(())
}

async fn get_versions(nanuak_dictionary_root_dir: impl AsRef<Path>) -> Vec<PathBuf> {
    // Simply return the child directories sorted by name.
    let mut versions = fs::read_dir(nanuak_dictionary_root_dir)
        .await?
        .filter_map(|entry| async {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .await;
}

async fn validate(nanuak_dictionary_root_dir: impl AsRef<Path>) -> bool {
    // Return true if the path is a directory with the exact name "Nanuak-Dictionary"
    let path = nanuak_dictionary_root_dir.as_ref();
    path.is_dir() && path.file_name() == Some(OsStr::new("Nanuak-Dictionary"))
}

async fn get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir() -> Result<PathBuf> {
    let current_dir = tokio::fs::canonicalize(".").await?;
    if validate(&current_dir).await {
        Ok(current_dir)
    } else {
        let parent_dir = current_dir.parent().ok_or_else(|| eyre!("No parent directory"))?;
        if validate(&parent_dir).await {
            Ok(parent_dir.to_path_buf())
        } else {
            Err(eyre!("No Nanuak-Dictionary directory found"))
        }
    }
}

async fn print_the_versions() -> Result<()> {
    let nanuak_dictionary_root_dir = get_nanuak_dictionary_root_dir_using_cwd_if_matches_or_parent_dir().await?;
    let versions = get_versions(&nanuak_dictionary_root_dir).await?;
    for version in versions {
        println!("{}", version.display());
    }
    Ok(())
}