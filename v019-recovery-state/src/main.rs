#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    tracing::info!("Ahoy!");
    v006_create_new_version::init().await?;
    v007_create_new_version::create_new_version().await?;
    tracing::info!("Goodbye from {}", env!("CARGO_PKG_NAME"));
    Ok(())
}
