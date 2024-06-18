use serde::{Deserialize, Serialize};

use crate::game::{Drawing, Prompt, SignedCombination, UnsignedCombination};

// Messages from clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { name: String },
    Disconnect,
    SubmitDrawing(Drawing),
    SubmitPrompt(Prompt),
    SubmitCombination(SignedCombination),
    SubmitVote(bool),
}

// Messages from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Draw {
        duration: u16,
    },
    Prompt {
        duration: u16,
    },
    Combine {
        duration: u16,
        drawings: Vec<Drawing>,
        prompts: Vec<Prompt>,
    },
    Vote {
        duration: u16,
        drawing1: UnsignedCombination,
        drawing2: UnsignedCombination,
    },
    Wait {
        message: String,
    },
}
