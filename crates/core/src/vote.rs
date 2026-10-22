//! Final ballots and the aggregate verdict.

use serde::{Deserialize, Serialize};

use crate::debate::{DebateBatch, DebateReply};
use crate::opening::{Opening, OpeningBatch};
use crate::persona::PersonaId;
use crate::stage::Stance;

/// What a provider returns before the usecase attaches the opening baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct VoteCast {
    pub persona: PersonaId,
    pub stance: Stance,
    pub confidence: u8,
    pub reason: String,
    pub model: String,
}

/// A final ballot enriched with the brain's opening stance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalVote {
    pub persona: PersonaId,
    pub stance: Stance,
    pub confidence: u8,
    pub reason: String,
    pub model: String,
    pub opening_stance: Option<Stance>,
    /// `Some(true)` changed, `Some(false)` stable, `None` no opening baseline.
    pub changed_from_opening: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VoteOutcome {
    Ok { vote: FinalVote },
    Failed { persona: PersonaId, message: String },
}

impl VoteOutcome {
    pub fn persona(&self) -> PersonaId {
        match self {
            Self::Ok { vote } => vote.persona,
            Self::Failed { persona, .. } => *persona,
        }
    }

    pub fn vote(&self) -> Option<&FinalVote> {
        match self {
            Self::Ok { vote } => Some(vote),
            Self::Failed { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteTally {
    pub approve: u8,
    pub reject: u8,
    pub abstain: u8,
    pub failed: u8,
}

impl VoteTally {
    pub fn from_outcomes(outcomes: &[VoteOutcome]) -> Self {
        let mut tally = Self::default();
        for outcome in outcomes {
            match outcome {
                VoteOutcome::Ok { vote } => match vote.stance {
                    Stance::Approve => tally.approve += 1,
                    Stance::Reject => tally.reject += 1,
                    Stance::Abstain => tally.abstain += 1,
                },
                VoteOutcome::Failed { .. } => tally.failed += 1,
            }
        }
        tally
    }

    pub fn total(&self) -> u8 {
        self.approve + self.reject + self.abstain + self.failed
    }
}

/// The protocol's final decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MagiVerdict {
    Unanimous { stance: Stance },
    Majority { stance: Stance },
    Split,
    Incomplete,
}

/// Always exactly three outcomes, ordered like [`PersonaId::ALL`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteBatch {
    pub outcomes: Vec<VoteOutcome>,
    pub tally: VoteTally,
    pub verdict: MagiVerdict,
    pub prompt_version: String,
}

impl VoteBatch {
    pub const PROMPT_VERSION: &'static str = "v1";

    pub fn vote(&self, persona: PersonaId) -> Option<&FinalVote> {
        self.outcomes.iter().find_map(|outcome| match outcome {
            VoteOutcome::Ok { vote } if vote.persona == persona => Some(vote),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteRequest {
    pub question: String,
    #[serde(default)]
    pub context: String,
    pub openings: OpeningBatch,
    #[serde(default)]
    pub debate: Option<DebateBatch>,
}

/// Adapter-facing input. It deliberately contains no other final ballots.
#[derive(Debug, Clone, PartialEq)]
pub struct VoteCall {
    pub persona: PersonaId,
    pub question: String,
    pub context: String,
    pub own_opening: Option<Opening>,
    pub peer_openings: Vec<Opening>,
    pub debate: Vec<DebateReply>,
}
