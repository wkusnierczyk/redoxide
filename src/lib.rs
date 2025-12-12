pub mod evaluator;
pub mod runner;
pub mod strategy;
pub mod target;

use serde::{Deserialize, Serialize};

/// A convenient alias for Results in this crate
pub type RedOxideResult<T> = anyhow::Result<T>;

/// The result of a single red team attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResult {
    pub prompt: String,
    pub response: String,
    pub success: bool, // True if the model was successfully attacked
    pub strategy_name: String,
}
