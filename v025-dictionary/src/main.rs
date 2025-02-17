use v025_dictionary::Action;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    tracing::info!("Ahoy!");
    v006_create_new_version::init().await?;
    let action = Action::prompt_user_to_pick_an_action().await?;
    action.perform().await?;
    tracing::info!("Goodbye from {}", env!("CARGO_PKG_NAME"));
    Ok(())
}
