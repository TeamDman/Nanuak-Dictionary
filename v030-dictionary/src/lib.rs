pub mod state;
pub mod define_word_state;
pub mod create_new_version_state;

use strum::VariantArray;

#[derive(Debug, VariantArray)]
pub enum Action {
    CreateNewVersion,
    DefineWord,
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateNewVersion => write!(f, "Create a new version"),
            Self::DefineWord => write!(f, "Define a word"),
        }
    }
}
impl Action {
    pub async fn prompt_user_to_pick_an_action() -> eyre::Result<&'static Action> {
        let actions = Action::VARIANTS;
        let action = dialoguer::Select::new()
            .with_prompt("Choose an action")
            .items(&actions)
            .default(0)
            .interact()?;
        Ok(&actions[action])
    }
}

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
