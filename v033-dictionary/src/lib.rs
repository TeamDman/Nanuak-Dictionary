pub mod state;
pub mod define_word_state;
pub mod create_new_version_state;

pub async fn create_new_version() -> eyre::Result<()> {
    tracing::info!("Creating a new version");
    v007_create_new_version::create_new_version().await?;
    Ok(())
}

pub async fn define_word(_word: &str) -> eyre::Result<()> {
    tracing::info!("Defining a word");
    let _definition = todo!("get word definition");
    // println!("{}: {}", word, definition);
    // Ok(())
}

pub async fn prompt_user_for_word() -> eyre::Result<String> {
    tracing::info!("Prompting the user for a word");
    let word = dialoguer::Input::new()
        .with_prompt("Enter a word")
        .interact()?;
    Ok(word)
}
