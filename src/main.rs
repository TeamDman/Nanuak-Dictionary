use std::time::Duration;

use eyre::OptionExt;
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init()?;
    info!("Hi!");
    let ollama = Ollama::new("http://host.docker.internal".to_string(), 11434);
    let model = "phi4:14b";
    let prompt = "Why is the sky blue?";
    let request = GenerationRequest::new(model.to_string(), prompt);
    info!("Request: {:?}", request);
    let response = ollama.generate(request).await?;
    info!("Response: {:?}", response.response);
    info!(
        "Took {} ms",
        Duration::from_millis(
            response
                .total_duration
                .ok_or_eyre("Expected total duration to be present")?
        )
        .as_millis()
    );
    Ok(())
}
