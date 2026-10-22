//! Protocol stages, ballot stances, and streamed telemetry events.

use serde::{Deserialize, Serialize};

use crate::persona::PersonaId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Opening,
    Debate,
    Vote,
}

impl Stage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opening => "opening",
            Self::Debate => "debate",
            Self::Vote => "vote",
        }
    }
}

/// A brain's disposition toward the question. Used for both the opening and
/// the final ballot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    Approve,
    Reject,
    Abstain,
}

impl Stance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Abstain => "abstain",
        }
    }

    pub const fn is_decisive(self) -> bool {
        !matches!(self, Self::Abstain)
    }
}

/// Telemetry sent over the Tauri channel while a stage runs.
///
/// These events are never the source of truth: the command's return value is
/// the authoritative batch. Consumers must tolerate interleaving across
/// personas and ignore stale `run_id`s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MagiStreamEvent {
    StageStarted {
        run_id: String,
        stage: Stage,
        personas: Vec<PersonaId>,
    },
    PersonaStarted {
        run_id: String,
        stage: Stage,
        persona: PersonaId,
    },
    Delta {
        run_id: String,
        stage: Stage,
        persona: PersonaId,
        text: String,
    },
    PersonaCompleted {
        run_id: String,
        stage: Stage,
        persona: PersonaId,
    },
    PersonaFailed {
        run_id: String,
        stage: Stage,
        persona: PersonaId,
        message: String,
    },
    StageCompleted {
        run_id: String,
        stage: Stage,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stance_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&Stance::Abstain).unwrap(),
            "\"abstain\""
        );
    }

    #[test]
    fn stream_events_are_tagged_by_kind() {
        let event = MagiStreamEvent::Delta {
            run_id: "r1".into(),
            stage: Stage::Opening,
            persona: PersonaId::Casper,
            text: "{".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["kind"], "delta");
        assert_eq!(json["stage"], "opening");
        assert_eq!(json["persona"], "casper");
        assert_eq!(json["run_id"], "r1");
    }
}
