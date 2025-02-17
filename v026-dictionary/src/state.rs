use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use serde::Deserialize;
use serde::Serialize;
use tracing::info;

use crate::create_new_version_state::CreateNewVersionState;
use crate::define_word_state::DefineWordState;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum DictionaryApplicationState {
    #[default]
    JustLaunchedNoArgs,
    DefineWord(DefineWordState),
    CreateNewVersion(CreateNewVersionState),
    Done,
}
const INITIAL_ACTIONS: [DictionaryApplicationState; 2] = [
    DictionaryApplicationState::DefineWord(DefineWordState::PromptingForWordToDefine),
    DictionaryApplicationState::CreateNewVersion(
        CreateNewVersionState::DetermineWorkspaceCargoTomlPath,
    ),
];

#[async_trait::async_trait]
pub trait State: Sized {
    fn describe(&self) -> &'static str;
    async fn next(self) -> eyre::Result<Self>
    where
        Self: Sized;
    fn is_terminal(&self) -> bool;

    async fn next_until_terminal(self) -> eyre::Result<Self> {
        let mut state = self;
        loop {
            info!("Applying state: {}", state.describe());
            state = state.next().await?;
            info!("Next state: {}", state.describe());
            if state.is_terminal() {
                break;
            }
        }
        Ok(state)
    }
}

#[async_trait::async_trait]
impl State for DictionaryApplicationState {
    fn describe(&self) -> &'static str {
        match self {
            Self::JustLaunchedNoArgs => "Start the application",
            Self::DefineWord(state) => state.describe(),
            Self::CreateNewVersion(state) => state.describe(),
            Self::Done => "Done",
        }
    }
    async fn next(self) -> eyre::Result<Self> {
        match self {
            Self::JustLaunchedNoArgs => {
                let chosen = pick(FzfArgs {
                    choices: INITIAL_ACTIONS
                        .iter()
                        .map(|action| Choice {
                            key: action.describe().to_string(),
                            value: action,
                        })
                        .collect(),
                    header: Some("Choose an action".to_string()),
                    prompt: None,
                })?;
                Ok(chosen.value.clone())
            }
            Self::DefineWord(state) => {
                _ = state.next_until_terminal().await?;
                Ok(Self::Done)
            }
            Self::CreateNewVersion(state) => {
                _ = state.next_until_terminal().await?;
                Ok(Self::Done)
            }
            Self::Done => Ok(Self::Done),
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}
