use v035_dictionary::state::State;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    tracing::info!("Ahoy!");
    v006_create_new_version::init().await?;
    let mut state = v035_dictionary::state::DictionaryApplicationState::default();
    loop {
        tracing::info!("Current state: {}", state.describe());
        state = state.next().await?;
        if state.is_terminal() {
            break;
        }
    }
    tracing::info!("Goodbye from {}", env!("CARGO_PKG_NAME"));
    Ok(())
}
