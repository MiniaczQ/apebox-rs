//! Game related structures.

use std::fmt::Debug;

use bevy::prelude::*;
use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

/// Single drawing with predetermined size.
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Drawing {
    pub data: Vec<u8>,
}

impl Debug for Drawing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drawing")
            .field("data", &self.data.len())
            .finish()
    }
}

/// Single user prompt.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub data: String,
}

/// Combination of a prompt and a drawing.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Combination {
    pub drawing: u64,
    pub prompt: u64,
}

/// Vote for a combination.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub combination: u64,
}

/// Author of a submission
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Network id, can go invalid.
    pub id: ClientId,
    /// Human readable name as a backup.
    pub name: String,
}

/// Marker for drawing/prompt that's already combined.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Combined;

/// Serde-able index.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Index(pub u64);

/// Index generator.
#[derive(Resource, Debug, Default)]
pub struct Indexer(u64);

impl Indexer {
    pub fn next(&mut self) -> Index {
        self.0 += 1;
        Index(self.0)
    }
}
