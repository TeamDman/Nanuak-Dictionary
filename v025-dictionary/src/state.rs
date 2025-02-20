use std::path::PathBuf;

use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use serde::Deserialize;
use serde::Serialize;

use crate::create_new_version;
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
    DictionaryApplicationState::BeginCreateNewVersion,
];

pub trait State {
    fn describe(&self) -> &'static str;
    async fn next(self) -> eyre::Result<Self>
    where
        Self: Sized;
    fn is_terminal(&self) -> bool;
}

impl State for DictionaryApplicationState {
    fn describe(&self) -> &'static str {
        match self {
            Self::JustLaunchedNoArgs => "Start the application",
            Self::BeginCreateNewVersion => "Create a new version",
            Self::Done => "Done",
            Self::DefineWord(state) => state.describe(),
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
                let next_state = state.next().await?;
                if next_state.is_terminal() {
                    Ok(Self::Done)
                } else {
                    Ok(Self::DefineWord(next_state))
                }
            }
            Self::BeginCreateNewVersion => {
                create_new_version().await?;
                Ok(Self::Done)
            }
            Self::Done => Ok(Self::Done),
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
}
