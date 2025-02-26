pub mod get_description;

use tracing::info;

fn init() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::DEBUG.into())
                .from_env()?,
        )
        .init();
    Ok(())
}

fn main() -> eyre::Result<()> {
    init()?;
    info!("Hi!");
    Ok(())
}
