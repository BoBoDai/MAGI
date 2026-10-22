//! Pure verdict aggregation.
//!
//! Abstentions never form a majority, and a failed ballot always yields
//! [`MagiVerdict::Incomplete`]. The UI renders dissent from the same tally.

use crate::stage::Stance;
use crate::vote::{MagiVerdict, VoteOutcome, VoteTally};

/// Aggregate exactly three ballots.
///
/// A slice with a different length is incomplete by construction.
pub fn aggregate_votes(outcomes: &[VoteOutcome]) -> (VoteTally, MagiVerdict) {
    let tally = VoteTally::from_outcomes(outcomes);

    if outcomes.len() != 3 || tally.failed > 0 {
        return (tally, MagiVerdict::Incomplete);
    }

    let verdict = match (tally.approve, tally.reject, tally.abstain) {
        (3, 0, 0) => MagiVerdict::Unanimous {
            stance: Stance::Approve,
        },
        (0, 3, 0) => MagiVerdict::Unanimous {
            stance: Stance::Reject,
        },
        (0, 0, 3) => MagiVerdict::Unanimous {
            stance: Stance::Abstain,
        },
        (2, _, _) => MagiVerdict::Majority {
            stance: Stance::Approve,
        },
        (_, 2, _) => MagiVerdict::Majority {
            stance: Stance::Reject,
        },
        // 1/1/1, two abstentions plus one decisive vote, or one decisive vote
        // plus two abstentions all fail to form a majority.
        _ => MagiVerdict::Split,
    };

    (tally, verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persona::PersonaId;
    use crate::vote::FinalVote;

    fn ballot(persona: PersonaId, stance: Stance) -> VoteOutcome {
        VoteOutcome::Ok {
            vote: FinalVote {
                persona,
                stance,
                confidence: 80,
                reason: "reason".into(),
                model: "test".into(),
                opening_stance: None,
                changed_from_opening: None,
            },
        }
    }

    fn ballot_set(stances: [Stance; 3]) -> Vec<VoteOutcome> {
        PersonaId::ALL
            .into_iter()
            .zip(stances)
            .map(|(persona, stance)| ballot(persona, stance))
            .collect()
    }

    #[test]
    fn three_approve_is_unanimous() {
        let (tally, verdict) = aggregate_votes(&ballot_set([
            Stance::Approve,
            Stance::Approve,
            Stance::Approve,
        ]));
        assert_eq!(tally.approve, 3);
        assert_eq!(
            verdict,
            MagiVerdict::Unanimous {
                stance: Stance::Approve
            }
        );
    }

    #[test]
    fn three_reject_is_unanimous() {
        let (_, verdict) = aggregate_votes(&ballot_set([
            Stance::Reject,
            Stance::Reject,
            Stance::Reject,
        ]));
        assert_eq!(
            verdict,
            MagiVerdict::Unanimous {
                stance: Stance::Reject
            }
        );
    }

    #[test]
    fn three_abstain_is_unanimous_without_recommendation() {
        let (_, verdict) = aggregate_votes(&ballot_set([
            Stance::Abstain,
            Stance::Abstain,
            Stance::Abstain,
        ]));
        assert_eq!(
            verdict,
            MagiVerdict::Unanimous {
                stance: Stance::Abstain
            }
        );
    }

    #[test]
    fn two_approve_one_reject_is_majority() {
        let (tally, verdict) = aggregate_votes(&ballot_set([
            Stance::Approve,
            Stance::Approve,
            Stance::Reject,
        ]));
        assert_eq!(tally.approve, 2);
        assert_eq!(
            verdict,
            MagiVerdict::Majority {
                stance: Stance::Approve
            }
        );
    }

    #[test]
    fn two_approve_one_abstain_is_majority() {
        let (_, verdict) = aggregate_votes(&ballot_set([
            Stance::Approve,
            Stance::Abstain,
            Stance::Approve,
        ]));
        assert_eq!(
            verdict,
            MagiVerdict::Majority {
                stance: Stance::Approve
            }
        );
    }

    #[test]
    fn two_reject_one_approve_is_majority() {
        let (_, verdict) = aggregate_votes(&ballot_set([
            Stance::Reject,
            Stance::Reject,
            Stance::Approve,
        ]));
        assert_eq!(
            verdict,
            MagiVerdict::Majority {
                stance: Stance::Reject
            }
        );
    }

    #[test]
    fn two_abstain_and_one_decisive_is_split() {
        let (tally, verdict) = aggregate_votes(&ballot_set([
            Stance::Abstain,
            Stance::Abstain,
            Stance::Approve,
        ]));
        assert_eq!(tally.abstain, 2);
        assert_eq!(verdict, MagiVerdict::Split);
    }

    #[test]
    fn one_of_each_is_split() {
        let (_, verdict) = aggregate_votes(&ballot_set([
            Stance::Approve,
            Stance::Reject,
            Stance::Abstain,
        ]));
        assert_eq!(verdict, MagiVerdict::Split);
    }

    #[test]
    fn failed_ballot_is_incomplete_even_with_two_approvals() {
        let mut ballots = ballot_set([Stance::Approve, Stance::Approve, Stance::Approve]);
        ballots[2] = VoteOutcome::Failed {
            persona: PersonaId::Casper,
            message: "timeout".into(),
        };
        let (tally, verdict) = aggregate_votes(&ballots);
        assert_eq!(tally.approve, 2);
        assert_eq!(tally.failed, 1);
        assert_eq!(verdict, MagiVerdict::Incomplete);
    }

    #[test]
    fn wrong_length_is_incomplete() {
        let ballots = ballot_set([Stance::Approve, Stance::Approve, Stance::Approve]);
        let (_, verdict) = aggregate_votes(&ballots[..2]);
        assert_eq!(verdict, MagiVerdict::Incomplete);
    }
}
