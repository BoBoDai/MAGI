//! Opening statements: three independent first opinions.

use serde::{Deserialize, Serialize};

use crate::persona::PersonaId;
use crate::stage::Stance;

/// One brain's structured opening statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opening {
    pub persona: PersonaId,
    pub stance: Stance,
    /// 0-100, validated during parsing.
    pub confidence: u8,
    pub summary: String,
    pub key_points: Vec<String>,
    pub risks: Vec<String>,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OpeningOutcome {
    Ok { opening: Opening },
    Failed { persona: PersonaId, message: String },
}

impl OpeningOutcome {
    pub fn persona(&self) -> PersonaId {
        match self {
            Self::Ok { opening } => opening.persona,
            Self::Failed { persona, .. } => *persona,
        }
    }

    pub fn opening(&self) -> Option<&Opening> {
        match self {
            Self::Ok { opening } => Some(opening),
            Self::Failed { .. } => None,
        }
    }
}

/// Always exactly three outcomes, ordered like [`PersonaId::ALL`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpeningBatch {
    pub outcomes: Vec<OpeningOutcome>,
    pub prompt_version: String,
}

impl OpeningBatch {
    pub const PROMPT_VERSION: &'static str = "v1";

    pub fn all_succeeded(&self) -> bool {
        self.outcomes.len() == PersonaId::ALL.len()
            && self
                .outcomes
                .iter()
                .all(|outcome| matches!(outcome, OpeningOutcome::Ok { .. }))
    }

    pub fn opening(&self, persona: PersonaId) -> Option<&Opening> {
        self.outcomes.iter().find_map(|outcome| match outcome {
            OpeningOutcome::Ok { opening } if opening.persona == persona => Some(opening),
            _ => None,
        })
    }

    pub fn successful_openings(&self) -> Vec<&Opening> {
        self.outcomes
            .iter()
            .filter_map(OpeningOutcome::opening)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpeningRequest {
    pub run_id: String,
    pub question: String,
    #[serde(default)]
    pub context: String,
}

/// Adapter-facing input for a single brain. No peer output is included.
#[derive(Debug, Clone, PartialEq)]
pub struct OpeningCall {
    pub persona: PersonaId,
    pub question: String,
    pub context: String,
}
