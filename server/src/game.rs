use std::time::Duration;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateData {
    Draw(DrawConfig),
    Prompt(PromptConfig),
    Combine(CombineConfig),
    Vote(VoteConfig),
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrawConfig {
    pub duration: Duration,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptConfig {
    pub prompts_per_player: usize,
    pub duration: Duration,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombineConfig {
    pub duration: Duration,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoteConfig {
    pub duration: Duration,
}

/// Game configuration.
#[derive(Resource)]
pub struct GameConfig {
    pub extra_time: Duration,
    pub states: Vec<StateData>,
}

impl GameConfig {
    pub fn short() -> Self {
        Self {
            extra_time: Duration::from_secs(30),
            states: vec![
                StateData::Vote(VoteConfig {
                    duration: Duration::from_secs(4),
                }),
                StateData::Combine(CombineConfig {
                    duration: Duration::from_secs(4),
                }),
                StateData::Prompt(PromptConfig {
                    prompts_per_player: 3,
                    duration: Duration::from_secs(4),
                }),
                StateData::Draw(DrawConfig {
                    duration: Duration::from_secs(4),
                }),
                StateData::Draw(DrawConfig {
                    duration: Duration::from_secs(4),
                }),
            ],
        }
    }

    pub fn next_state(&mut self) -> Option<StateData> {
        self.states.pop()
    }
}
