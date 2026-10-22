//! Persisted session records and their integrity rules.
//!
//! The frontend sends a [`SessionDraft`] back after each stage. The store must
//! never trust the draft's verdict, tally, or changed-stance flags: they are
//! recomputed here before the record is written.

use serde::{Deserialize, Serialize};

use crate::debate::DebateBatch;
use crate::decision::decide_openings;
use crate::error::{Error, Result};
use crate::opening::{OpeningBatch, OpeningOutcome};
use crate::persona::PersonaId;
use crate::verdict::aggregate_votes;
use crate::vote::{MagiVerdict, VoteBatch, VoteOutcome, VoteTally};

pub const SESSION_SCHEMA_VERSION: u32 = 1;
pub const MAX_STORED_SESSIONS: usize = 100;
pub const SESSION_QUESTION_MAX_CHARS: usize = 4_000;
pub const SESSION_CONTEXT_MAX_CHARS: usize = 20_000;
pub const SESSION_ID_MAX_CHARS: usize = 128;

/// IPC input for saving a session. Timestamps and schema version are owned by
/// the backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionDraft {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub openings: Option<OpeningBatch>,
    #[serde(default)]
    pub debate: Option<DebateBatch>,
    #[serde(default)]
    pub vote: Option<VoteBatch>,
}

/// Where the session's model output came from. Mock sessions are only
/// possible in debug builds, but the marker is persisted so history can never
/// present fake output as real.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionSource {
    Live,
    Mock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub schema_version: u32,
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub source: SessionSource,
    pub question: String,
    pub context: String,
    pub openings: Option<OpeningBatch>,
    pub debate: Option<DebateBatch>,
    pub vote: Option<VoteBatch>,
}

impl SessionRecord {
    pub fn from_draft(
        draft: SessionDraft,
        source: SessionSource,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            schema_version: SESSION_SCHEMA_VERSION,
            id: draft.id,
            created_at,
            updated_at,
            source,
            question: draft.question,
            context: draft.context,
            openings: draft.openings,
            debate: draft.debate,
            vote: draft.vote,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Only the opening stage is saved.
    Opening,
    /// The debate round is saved, but not the final ballot.
    Debate,
    /// The final ballot and verdict are saved.
    Vote,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub question: String,
    pub status: SessionStatus,
    pub verdict: Option<MagiVerdict>,
    pub tally: Option<VoteTally>,
    pub opening_failures: u8,
    pub debate_failures: u8,
    pub source: SessionSource,
}

impl SessionSummary {
    pub fn from_record(record: &SessionRecord) -> Self {
        let status = if record.vote.is_some() {
            SessionStatus::Vote
        } else if record.debate.is_some() {
            SessionStatus::Debate
        } else {
            SessionStatus::Opening
        };

        Self {
            id: record.id.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            question: record.question.clone(),
            status,
            verdict: record.vote.as_ref().map(|vote| vote.verdict.clone()),
            tally: record.vote.as_ref().map(|vote| vote.tally),
            opening_failures: record
                .openings
                .as_ref()
                .map(|batch| {
                    batch
                        .outcomes
                        .iter()
                        .filter(|outcome| matches!(outcome, OpeningOutcome::Failed { .. }))
                        .count() as u8
                })
                .unwrap_or(0),
            debate_failures: record
                .debate
                .as_ref()
                .map(|batch| {
                    batch
                        .outcomes
                        .iter()
                        .filter(|outcome| {
                            matches!(outcome, crate::debate::DebateOutcome::Failed { .. })
                        })
                        .count() as u8
                })
                .unwrap_or(0),
            source: record.source,
        }
    }
}

impl SessionDraft {
    /// Validate the draft and recompute every derived field.
    pub fn recompute(&mut self) -> Result<()> {
        self.validate()?;

        if let Some(openings) = self.openings.as_mut() {
            openings.decision = decide_openings(&openings.outcomes);
        }

        let Some(vote) = self.vote.as_mut() else {
            return Ok(());
        };

        let baselines = PersonaId::ALL.map(|persona| {
            self.openings
                .as_ref()
                .and_then(|batch| batch.opening(persona))
                .map(|opening| opening.stance)
        });

        for outcome in &mut vote.outcomes {
            if let VoteOutcome::Ok { vote } = outcome {
                let index = PersonaId::ALL
                    .iter()
                    .position(|persona| *persona == vote.persona)
                    .unwrap_or(0);
                let baseline = baselines[index];
                vote.opening_stance = baseline;
                vote.changed_from_opening = baseline.map(|from| from != vote.stance);
            }
        }

        let (tally, verdict) = aggregate_votes(&vote.outcomes);
        vote.tally = tally;
        vote.verdict = verdict;

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::InvalidInput("session id is empty".into()));
        }
        if self.id.chars().count() > SESSION_ID_MAX_CHARS {
            return Err(Error::InvalidInput(format!(
                "session id exceeds {SESSION_ID_MAX_CHARS} characters"
            )));
        }
        if self.question.trim().is_empty() {
            return Err(Error::InvalidInput("question is empty".into()));
        }
        if self.question.chars().count() > SESSION_QUESTION_MAX_CHARS {
            return Err(Error::InvalidInput(format!(
                "question exceeds {SESSION_QUESTION_MAX_CHARS} characters"
            )));
        }
        if self.context.chars().count() > SESSION_CONTEXT_MAX_CHARS {
            return Err(Error::InvalidInput(format!(
                "context exceeds {SESSION_CONTEXT_MAX_CHARS} characters"
            )));
        }

        let Some(openings) = &self.openings else {
            return Err(Error::InvalidInput(
                "a session must contain an opening batch".into(),
            ));
        };

        validate_slots("opening", &openings.outcomes, OpeningOutcome::persona)?;
        for outcome in &openings.outcomes {
            match outcome {
                OpeningOutcome::Ok { opening } => {
                    check_confidence(opening.confidence)?;
                    require_text("opening summary", &opening.summary, 1_200)?;
                    require_text("opening model", &opening.model, 200)?;
                    check_bullets("opening key_points", &opening.key_points, 1, 8)?;
                    check_bullets("opening risks", &opening.risks, 1, 8)?;
                }
                OpeningOutcome::Failed { message, .. } => {
                    require_text("opening failure message", message, 1_000)?;
                }
            }
        }

        if let Some(debate) = &self.debate {
            validate_slots(
                "debate",
                &debate.outcomes,
                crate::debate::DebateOutcome::persona,
            )?;
            for outcome in &debate.outcomes {
                match outcome {
                    crate::debate::DebateOutcome::Ok { reply } => {
                        require_text("debate reply", &reply.text, 6_000)?;
                        require_text("debate model", &reply.model, 200)?;
                    }
                    crate::debate::DebateOutcome::Failed { message, .. } => {
                        require_text("debate failure message", message, 1_000)?;
                    }
                }
            }
        }

        if let Some(vote) = &self.vote {
            validate_slots("vote", &vote.outcomes, VoteOutcome::persona)?;
            for outcome in &vote.outcomes {
                match outcome {
                    VoteOutcome::Ok { vote } => {
                        check_confidence(vote.confidence)?;
                        require_text("vote reason", &vote.reason, 800)?;
                        require_text("vote model", &vote.model, 200)?;
                    }
                    VoteOutcome::Failed { message, .. } => {
                        require_text("vote failure message", message, 1_000)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn validate_slots<T>(label: &str, outcomes: &[T], persona: impl Fn(&T) -> PersonaId) -> Result<()> {
    if outcomes.len() != PersonaId::ALL.len() {
        return Err(Error::InvalidInput(format!(
            "{label} must contain exactly {} outcomes",
            PersonaId::ALL.len()
        )));
    }

    for (index, expected) in PersonaId::ALL.iter().enumerate() {
        let actual = outcomes.get(index).map(&persona).unwrap_or(*expected);
        if actual != *expected {
            return Err(Error::InvalidInput(format!(
                "{label} slot {index} must be {}, got {}",
                expected.as_str(),
                actual.as_str()
            )));
        }
    }

    Ok(())
}

fn check_confidence(confidence: u8) -> Result<()> {
    if confidence > 100 {
        return Err(Error::InvalidInput(format!(
            "confidence {confidence} is out of range (0-100)"
        )));
    }
    Ok(())
}

fn require_text(label: &str, value: &str, max_chars: usize) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::InvalidInput(format!("{label} is empty")));
    }
    if value.chars().count() > max_chars {
        return Err(Error::InvalidInput(format!(
            "{label} exceeds {max_chars} characters"
        )));
    }
    Ok(())
}

fn check_bullets(label: &str, items: &[String], min: usize, max: usize) -> Result<()> {
    if items.len() < min || items.len() > max {
        return Err(Error::InvalidInput(format!(
            "{label} must contain between {min} and {max} items"
        )));
    }
    for item in items {
        require_text(label, item, 300)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opening::Opening;
    use crate::stage::Stance;
    use crate::vote::{FinalVote, VoteOutcome};

    fn opening(persona: PersonaId, stance: Stance) -> Opening {
        Opening {
            persona,
            stance,
            confidence: 70,
            summary: "summary".into(),
            key_points: vec!["point".into()],
            risks: vec!["risk".into()],
            model: "test".into(),
        }
    }

    fn openings(stances: [Stance; 3]) -> OpeningBatch {
        let outcomes: Vec<_> = PersonaId::ALL
            .into_iter()
            .zip(stances)
            .map(|(persona, stance)| OpeningOutcome::Ok {
                opening: opening(persona, stance),
            })
            .collect();
        let decision = crate::decision::decide_openings(&outcomes);

        OpeningBatch {
            outcomes,
            prompt_version: OpeningBatch::PROMPT_VERSION.into(),
            decision,
        }
    }

    fn final_vote(persona: PersonaId, stance: Stance) -> FinalVote {
        FinalVote {
            persona,
            stance,
            confidence: 80,
            reason: "reason".into(),
            model: "test".into(),
            opening_stance: None,
            changed_from_opening: None,
        }
    }

    fn vote_batch(stances: [Stance; 3]) -> VoteBatch {
        VoteBatch {
            outcomes: PersonaId::ALL
                .into_iter()
                .zip(stances)
                .map(|(persona, stance)| VoteOutcome::Ok {
                    vote: final_vote(persona, stance),
                })
                .collect(),
            tally: VoteTally::default(),
            verdict: MagiVerdict::Incomplete,
            prompt_version: VoteBatch::PROMPT_VERSION.into(),
        }
    }

    fn draft() -> SessionDraft {
        SessionDraft {
            id: "s1".into(),
            question: "Should we ship?".into(),
            context: String::new(),
            openings: Some(openings([Stance::Approve, Stance::Reject, Stance::Approve])),
            debate: None,
            vote: None,
        }
    }

    #[test]
    fn recompute_fills_tally_verdict_and_changes() {
        let mut draft = draft();
        draft.vote = Some(vote_batch([
            Stance::Approve,
            Stance::Approve,
            Stance::Reject,
        ]));

        draft.recompute().unwrap();

        let vote = draft.vote.as_ref().unwrap();
        assert_eq!(
            vote.tally,
            VoteTally {
                approve: 2,
                reject: 1,
                abstain: 0,
                failed: 0
            }
        );
        assert_eq!(
            vote.verdict,
            MagiVerdict::Majority {
                stance: Stance::Approve
            }
        );
        assert_eq!(
            vote.vote(PersonaId::Melchior).unwrap().changed_from_opening,
            Some(false)
        );
        assert_eq!(
            vote.vote(PersonaId::Balthasar)
                .unwrap()
                .changed_from_opening,
            Some(true)
        );
        assert_eq!(
            vote.vote(PersonaId::Casper).unwrap().changed_from_opening,
            Some(true)
        );
    }

    #[test]
    fn rejects_out_of_order_slots() {
        let mut draft = draft();
        let batch = draft.openings.as_mut().unwrap();
        batch.outcomes.swap(0, 1);

        assert!(matches!(draft.recompute(), Err(Error::InvalidInput(_))));
    }

    #[test]
    fn rejects_debate_without_openings() {
        let mut draft = draft();
        draft.openings = None;
        draft.debate = Some(DebateBatch {
            outcomes: vec![],
            prompt_version: "v1".into(),
        });

        assert!(matches!(draft.recompute(), Err(Error::InvalidInput(_))));
    }

    #[test]
    fn rejects_empty_question() {
        let mut draft = draft();
        draft.question = "  ".into();
        assert!(matches!(draft.recompute(), Err(Error::InvalidInput(_))));
    }

    #[test]
    fn summary_reports_latest_stage() {
        let mut draft = draft();
        draft.vote = Some(vote_batch([Stance::Reject, Stance::Reject, Stance::Reject]));
        draft.recompute().unwrap();

        let record = SessionRecord::from_draft(
            draft,
            SessionSource::Live,
            "2026-10-22T10:00:00Z".into(),
            "2026-10-22T10:01:00Z".into(),
        );
        let summary = SessionSummary::from_record(&record);

        assert_eq!(summary.status, SessionStatus::Vote);
        assert_eq!(
            summary.verdict,
            Some(MagiVerdict::Unanimous {
                stance: Stance::Reject
            })
        );
        assert_eq!(summary.tally.unwrap().reject, 3);
        assert_eq!(summary.opening_failures, 0);
    }

    #[test]
    fn source_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&SessionSource::Mock).unwrap(),
            "\"mock\""
        );
    }
}
