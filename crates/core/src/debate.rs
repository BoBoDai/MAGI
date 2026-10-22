//! The optional one-round cross-examination.

use serde::{Deserialize, Serialize};

use crate::opening::{Opening, OpeningBatch};
use crate::persona::PersonaId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebateReply {
    pub persona: PersonaId,
    /// Plain text, not JSON.
    pub text: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DebateOutcome {
    Ok { reply: DebateReply },
    Failed { persona: PersonaId, message: String },
}

impl DebateOutcome {
    pub fn persona(&self) -> PersonaId {
        match self {
            Self::Ok { reply } => reply.persona,
            Self::Failed { persona, .. } => *persona,
        }
    }

    pub fn reply(&self) -> Option<&DebateReply> {
        match self {
            Self::Ok { reply } => Some(reply),
            Self::Failed { .. } => None,
        }
    }
}

/// Always exactly three outcomes, ordered like [`PersonaId::ALL`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebateBatch {
    pub outcomes: Vec<DebateOutcome>,
    pub prompt_version: String,
}

impl DebateBatch {
    pub const PROMPT_VERSION: &'static str = "v1";

    pub fn all_succeeded(&self) -> bool {
        self.outcomes.len() == PersonaId::ALL.len()
            && self
                .outcomes
                .iter()
                .all(|outcome| matches!(outcome, DebateOutcome::Ok { .. }))
    }

    pub fn reply(&self, persona: PersonaId) -> Option<&DebateReply> {
        self.outcomes.iter().find_map(|outcome| match outcome {
            DebateOutcome::Ok { reply } if reply.persona == persona => Some(reply),
            _ => None,
        })
    }

    pub fn successful_replies(&self) -> Vec<&DebateReply> {
        self.outcomes
            .iter()
            .filter_map(DebateOutcome::reply)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebateRequest {
    pub run_id: String,
    pub question: String,
    #[serde(default)]
    pub context: String,
    pub openings: OpeningBatch,
}

/// Adapter-facing input. Contains no peer debate replies.
#[derive(Debug, Clone, PartialEq)]
pub struct DebateCall {
    pub persona: PersonaId,
    pub question: String,
    pub context: String,
    pub own_opening: Opening,
    pub peer_openings: Vec<Opening>,
}
