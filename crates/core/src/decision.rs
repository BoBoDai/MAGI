//! MAGI decision status derived from the three opening statements.
//!
//! This mirrors the console vocabulary of the classic MAGI decision system:
//! `合意` (yes), `拒絶` (no), `状態` (conditional), `情報` (info), `誤差`
//! (error). The precedence is error > info > no > conditional > yes.

use serde::{Deserialize, Serialize};

use crate::opening::{OpeningBatch, OpeningOutcome};
use crate::stage::Stance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    /// No binary decision is available.
    #[default]
    Info,
    /// All three brains approve.
    Yes,
    /// At least one brain rejects.
    No,
    /// No rejection, but at least one abstention.
    Conditional,
    /// A brain failed or the batch is malformed.
    Error,
}

/// Aggregate opening statements into a decision status.
pub fn decide_openings(outcomes: &[OpeningOutcome]) -> DecisionStatus {
    if outcomes.len() != 3 {
        return DecisionStatus::Error;
    }

    let mut stances = Vec::with_capacity(3);
    for outcome in outcomes {
        match outcome {
            OpeningOutcome::Ok { opening } => stances.push(opening.stance),
            OpeningOutcome::Failed { .. } => return DecisionStatus::Error,
        }
    }

    if stances.iter().all(|stance| *stance == Stance::Abstain) {
        return DecisionStatus::Info;
    }
    if stances.contains(&Stance::Reject) {
        return DecisionStatus::No;
    }
    if stances.contains(&Stance::Abstain) {
        return DecisionStatus::Conditional;
    }

    DecisionStatus::Yes
}

impl OpeningBatch {
    pub fn decision_status(&self) -> DecisionStatus {
        decide_openings(&self.outcomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opening::Opening;
    use crate::persona::PersonaId;

    fn batch(stances: [Stance; 3]) -> Vec<OpeningOutcome> {
        PersonaId::ALL
            .into_iter()
            .zip(stances)
            .map(|(persona, stance)| OpeningOutcome::Ok {
                opening: Opening {
                    persona,
                    stance,
                    confidence: 70,
                    summary: "summary".into(),
                    key_points: vec!["point".into()],
                    risks: vec!["risk".into()],
                    model: "test".into(),
                },
            })
            .collect()
    }

    #[test]
    fn all_approve_is_yes() {
        assert_eq!(
            decide_openings(&batch([Stance::Approve; 3])),
            DecisionStatus::Yes
        );
    }

    #[test]
    fn any_reject_is_no() {
        assert_eq!(
            decide_openings(&batch([Stance::Approve, Stance::Reject, Stance::Approve])),
            DecisionStatus::No
        );
    }

    #[test]
    fn abstention_without_reject_is_conditional() {
        assert_eq!(
            decide_openings(&batch([Stance::Approve, Stance::Abstain, Stance::Approve])),
            DecisionStatus::Conditional
        );
    }

    #[test]
    fn all_abstain_is_info() {
        assert_eq!(
            decide_openings(&batch([Stance::Abstain; 3])),
            DecisionStatus::Info
        );
    }

    #[test]
    fn failed_or_malformed_is_error() {
        let mut outcomes = batch([Stance::Approve; 3]);
        outcomes[1] = OpeningOutcome::Failed {
            persona: PersonaId::Balthasar,
            message: "timeout".into(),
        };
        assert_eq!(decide_openings(&outcomes), DecisionStatus::Error);
        assert_eq!(decide_openings(&outcomes[..2]), DecisionStatus::Error);
    }
}
