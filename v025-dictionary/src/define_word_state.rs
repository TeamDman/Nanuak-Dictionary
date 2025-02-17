use serde::Deserialize;
use serde::Serialize;

use crate::define_word;
use crate::prompt_user_for_word;
use crate::state::State;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum DefineWordState {
    #[default]
    PromptingForWordToDefine,
    DefiningWord { word: String },
    Done,
}
impl State for DefineWordState {
    fn describe(&self) -> &'static str {
        match self {
            Self::PromptingForWordToDefine => "Prompt me for a word to define",
            Self::DefiningWord { .. } => "Define a word",
            Self::Done => "Done",
        }
    }

    async fn next(self) -> eyre::Result<Self>
    where
        Self: Sized,
    {
        match self {
            Self::PromptingForWordToDefine => {
                let word = prompt_user_for_word().await?;
                Ok(Self::DefiningWord { word })
            }
            Self::DefiningWord { word } => {
                define_word(&word).await?;
                Ok(Self::Done)
            }
            Self::Done => Ok(Self::Done),
        }
    }
    
    fn is_terminal(&self) -> bool {
        matches!(self, Self::Done)
    }
    
}
