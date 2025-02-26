use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::io::BufRead;
use std::io::{self};
use std::time::Duration;
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
    info!("Ready. Type a word or phrase and press Enter to get a definition. (Ctrl+D to exit)");

    let ollama = Ollama::new("http://host.docker.internal".to_string(), 11434);
    let model = "phi4:14b";

    let stdin = io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        let word = line?;
        if word.trim().is_empty() {
            continue;
        }

        let prompt = format!("Define the term: {:?}", word);
        let request = GenerationRequest::new(model.to_string(), &prompt);
        info!("Sending request: {}", prompt);

        match ollama.generate(request).await {
            Ok(response) => {
                println!("Definition: {}", response.response);
                if let Some(duration) = response.total_duration {
                    info!(
                        "Response time: {} ms",
                        Duration::from_millis(duration).as_millis()
                    );
                }
            }
            Err(err) => {
                eprintln!("Error querying model: {:?}", err);
            }
        }
    }

    Ok(())
}
