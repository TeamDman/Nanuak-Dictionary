#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    v006_create_new_version::init().await?;
    v007_create_new_version::create_new_version().await?;
    Ok(())
}
