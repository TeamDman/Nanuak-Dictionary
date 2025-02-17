use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use serde::Deserialize;
use serde::Serialize;

use crate::create_new_version;
use crate::define_word;
use crate::prompt_user_for_word;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum DictionaryApplicationState {
    #[default]
    JustLaunchedNoArgs,
    PromptingForWordToDefine,
    DefiningWord {
        word: String,
    },
    CreatingNewVersion,
    Done,
}
const INITIAL_ACTIONS: [DictionaryApplicationState; 2] = [
    DictionaryApplicationState::PromptingForWordToDefine,
    DictionaryApplicationState::CreatingNewVersion,
];

impl DictionaryApplicationState {
    pub fn describe(&self) -> &'static str {
        match self {
            Self::JustLaunchedNoArgs => "Start the application",
            Self::PromptingForWordToDefine => "Prompt me for a word to define",
            Self::DefiningWord { .. } => "Define a word",
            Self::CreatingNewVersion => "Create a new version",
            Self::Done => "Done",
        }
    }
    pub async fn next(self) -> eyre::Result<Self> {
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
            Self::PromptingForWordToDefine => {
                let word = prompt_user_for_word().await?;
                Ok(Self::DefiningWord { word })
            }
            Self::DefiningWord { word } => {
                define_word(&word).await?;
                Ok(Self::Done)
            }
            Self::CreatingNewVersion => {
                create_new_version().await?;
                Ok(Self::Done)
            }
            Self::Done => Ok(Self::Done),
        }
    }
}
