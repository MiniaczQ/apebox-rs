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
    pub voting_duration: Duration,
    pub winner_duration: Duration,
}

/// Game configuration.
#[derive(Resource)]
pub struct GameConfig {
    pub extra_time: Duration,
    pub states: Vec<StateData>,
}

impl GameConfig {
    pub fn long() -> Self {
        Self {
            extra_time: Duration::from_secs(3),
            states: vec![
                StateData::Vote(VoteConfig {
                    voting_duration: Duration::from_secs(10),
                    winner_duration: Duration::from_secs(5),
                }),
                StateData::Combine(CombineConfig {
                    duration: Duration::from_secs(30),
                }),
                StateData::Prompt(PromptConfig {
                    prompts_per_player: 4,
                    duration: Duration::from_secs(60),
                }),
                StateData::Draw(DrawConfig {
                    duration: Duration::from_secs(180),
                }),
                StateData::Vote(VoteConfig {
                    voting_duration: Duration::from_secs(10),
                    winner_duration: Duration::from_secs(5),
                }),
                StateData::Combine(CombineConfig {
                    duration: Duration::from_secs(30),
                }),
                StateData::Prompt(PromptConfig {
                    prompts_per_player: 3,
                    duration: Duration::from_secs(60),
                }),
                StateData::Draw(DrawConfig {
                    duration: Duration::from_secs(180),
                }),
                StateData::Draw(DrawConfig {
                    duration: Duration::from_secs(180),
                }),
            ],
        }
    }

    pub fn next_state(&mut self) -> Option<StateData> {
        self.states.pop()
    }
}
