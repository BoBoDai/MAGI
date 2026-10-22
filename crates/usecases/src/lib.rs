//! # magi-usecases
//!
//! Transport-free protocol orchestration. No Tauri types belong here.
//!
//! Each stage runs the three brains concurrently with `tokio::join!` and
//! returns explicit per-persona outcomes. A failed brain is never dropped.

mod budget;
mod debate;
mod opening;
mod vote;

#[cfg(test)]
mod tests;

use magi_adapters::MagiProvider;
use magi_core::{DebateBatch, Error, OpeningBatch, PersonaId, Result};

/// Maximum characters accepted for the user's question.
pub const MAGI_QUESTION_BUDGET: usize = 2_000;
/// Maximum characters for the optional pasted context.
pub const MAGI_CONTEXT_BUDGET: usize = 40_000;
/// Maximum characters of any peer debate reply placed in a prompt.
pub const MAGI_PEER_BUDGET: usize = 6_000;

pub struct App<L> {
    pub llm: L,
}

impl<L: MagiProvider> App<L> {
    pub fn new(llm: L) -> Self {
        Self { llm }
    }
}

pub(crate) fn validate_openings(batch: &OpeningBatch) -> Result<()> {
    if batch.outcomes.len() != PersonaId::ALL.len() {
        return Err(Error::InvalidInput("开场必须包含三个脑".into()));
    }

    for (index, expected) in PersonaId::ALL.iter().enumerate() {
        let actual = batch.outcomes.get(index).map(|outcome| outcome.persona());
        if actual != Some(*expected) {
            return Err(Error::InvalidInput("开场顺序与脑的编号不一致".into()));
        }
    }

    Ok(())
}

pub(crate) fn validate_debate(batch: &DebateBatch) -> Result<()> {
    if batch.outcomes.len() != PersonaId::ALL.len() {
        return Err(Error::InvalidInput("辩论必须包含三个脑".into()));
    }

    for (index, expected) in PersonaId::ALL.iter().enumerate() {
        let actual = batch.outcomes.get(index).map(|outcome| outcome.persona());
        if actual != Some(*expected) {
            return Err(Error::InvalidInput("辩论顺序与脑的编号不一致".into()));
        }
    }

    Ok(())
}
