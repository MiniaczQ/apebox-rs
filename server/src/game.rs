use std::time::Duration;

use bevy::prelude::Resource;
use common::game::{Drawing, Prompt, Signed, SignedCombination};
use serde::{Deserialize, Serialize};

pub fn stages_short() -> Vec<GameStage> {
    vec![
        GameStage::Vote { duration: 5 },
        GameStage::Combine { duration: 30 },
        GameStage::Prompt {
            prompts_per_player: 3,
            duration: 30,
        },
        GameStage::Draw { duration: 120 },
        GameStage::Draw { duration: 120 },
    ]
}

/// A single stage of the game.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStage {
    /// Draw a single image.
    Draw { duration: u16 },
    /// Type multiple prompts.
    Prompt {
        prompts_per_player: u16,
        duration: u16,
    },
    /// Make a single combination of image and prompt.
    Combine { duration: u16 },
    /// Vote for the best combination.
    Vote { duration: u16 },
}

impl GameStage {
    pub fn duration(&self) -> Duration {
        match self {
            GameStage::Draw { duration } => Duration::from_secs(*duration as u64),
            GameStage::Prompt { duration, .. } => Duration::from_secs(*duration as u64),
            GameStage::Combine { duration } => Duration::from_secs(*duration as u64),
            GameStage::Vote { duration } => Duration::from_secs(*duration as u64),
        }
    }
}

/// Current state of the game.
#[derive(Resource)]
pub struct GameData {
    /// Stack of all following stages.
    pub next_stages: Vec<GameStage>,
    /// Current stage data.
    pub stage: Option<GameStage>,
    /// When the current stage ends.
    pub stage_end: Duration,
    /// Non-combined drawings.
    pub drawings: Vec<Signed<Drawing>>,
    /// Non-combined prompts.
    pub prompts: Vec<Signed<Prompt>>,
    /// Combined drawings and prompts.
    pub combinations: Vec<Signed<SignedCombination>>,
    /// Combinations past voting.
    pub voted: Vec<Signed<SignedCombination>>,
}

impl GameData {
    /// Create a game from stage definitions.
    pub fn from_stages(stages: Vec<GameStage>) -> Self {
        Self {
            next_stages: stages,
            stage: None,
            stage_end: Duration::ZERO,
            drawings: vec![],
            prompts: vec![],
            combinations: vec![],
            voted: vec![],
        }
    }

    /// Progress the game.
    pub fn try_progress(&mut self, now: Duration) -> bool {
        if self.stage_end < now {
            if let Some(next_stage) = self.next_stages.pop() {
                self.stage_end = now + next_stage.duration() + Duration::from_secs(5);
                self.stage = Some(next_stage);
            } else {
                self.stage_end = Duration::MAX;
                self.stage = None;
            }
            true
        } else {
            false
        }
    }

    /// Check if the game ended.
    pub fn has_ended(&self) -> bool {
        self.next_stages.is_empty() && self.stage.is_none()
    }
}
