//! Game related structures.

use serde::{Deserialize, Serialize};

/// Single drawing with predetermined size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing(pub Vec<u8>);

/// Single user prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt(pub String);

/// Combination of a drawing with a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCombination {
    pub drawing: Signed<Drawing>,
    pub prompt: Signed<Prompt>,
}

/// Combination of a drawing with a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedCombination {
    pub drawing: Drawing,
    pub prompt: Prompt,
}

/// Data labeled with an author.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signed<T> {
    pub data: T,
    pub author: String,
}
