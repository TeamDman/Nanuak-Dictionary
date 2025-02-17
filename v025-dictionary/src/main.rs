use v025_dictionary::{state::DictionaryApplicationState, Action};

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    tracing::info!("Ahoy!");
    v006_create_new_version::init().await?;
    let mut state: DictionaryApplicationState = Default::default();
    loop {
        state = state.next().await?;
        if state == DictionaryApplicationState::Done {
            break;
        }
    }
    tracing::info!("Goodbye from {}", env!("CARGO_PKG_NAME"));
    Ok(())
}
