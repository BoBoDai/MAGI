use magi_core::{MagiVerdict, PersonaId, Stance, VoteOutcome};

use super::support::{FakeProvider, debate_batch, opening_batch, vote_request};
use crate::App;

#[tokio::test]
async fn enriches_changed_stances() {
    let app = App::new(
        FakeProvider::default()
            .with_opening_stances([Stance::Approve, Stance::Reject, Stance::Approve])
            .with_final_stances([Stance::Approve, Stance::Approve, Stance::Approve]),
    );

    let batch = app
        .cast_votes(vote_request(
            opening_batch([Stance::Approve, Stance::Reject, Stance::Approve]),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        batch.verdict,
        MagiVerdict::Unanimous {
            stance: Stance::Approve
        }
    );

    let melchior = batch.vote(PersonaId::Melchior).unwrap();
    let balthasar = batch.vote(PersonaId::Balthasar).unwrap();
    let casper = batch.vote(PersonaId::Casper).unwrap();

    assert_eq!(melchior.changed_from_opening, Some(false));
    assert_eq!(balthasar.changed_from_opening, Some(true));
    assert_eq!(casper.changed_from_opening, Some(false));
}

#[tokio::test]
async fn failed_ballot_makes_the_verdict_incomplete() {
    let app = App::new(
        FakeProvider::default()
            .with_final_stances([Stance::Approve; 3])
            .failing_vote(PersonaId::Casper),
    );

    let batch = app
        .cast_votes(vote_request(opening_batch([Stance::Approve; 3]), None))
        .await
        .unwrap();

    assert_eq!(batch.verdict, MagiVerdict::Incomplete);
    assert!(matches!(batch.outcomes[2], VoteOutcome::Failed { .. }));
    assert_eq!(batch.tally.failed, 1);
}

#[tokio::test]
async fn produces_a_split_verdict() {
    let app = App::new(FakeProvider::default().with_final_stances([
        Stance::Approve,
        Stance::Reject,
        Stance::Abstain,
    ]));

    let batch = app
        .cast_votes(vote_request(opening_batch([Stance::Approve; 3]), None))
        .await
        .unwrap();

    assert_eq!(batch.verdict, MagiVerdict::Split);
}

#[tokio::test]
async fn isolates_each_ballot() {
    let app = App::new(FakeProvider::default());

    app.cast_votes(vote_request(opening_batch([Stance::Approve; 3]), None))
        .await
        .unwrap();

    let calls = app.llm.recorded_votes();
    assert_eq!(calls.len(), 3);
    for (index, call) in calls.iter().enumerate() {
        assert_eq!(call.persona, PersonaId::ALL[index]);
        assert_eq!(call.peer_openings.len(), 2);
        assert!(call.debate.is_empty());
        assert!(call.own_opening.is_some());
    }
    assert_eq!(app.llm.vote_calls(), 3);
}

#[tokio::test]
async fn includes_debate_replies_in_the_ballot_context() {
    let app = App::new(FakeProvider::default());

    app.cast_votes(vote_request(
        opening_batch([Stance::Approve; 3]),
        Some(debate_batch()),
    ))
    .await
    .unwrap();

    let calls = app.llm.recorded_votes();
    assert!(calls.iter().all(|call| call.debate.len() == 3));
}

#[tokio::test]
async fn rejects_out_of_order_debate() {
    let app = App::new(FakeProvider::default());
    let mut debate = debate_batch();
    debate.outcomes.swap(0, 2);

    let error = app
        .cast_votes(vote_request(
            opening_batch([Stance::Approve; 3]),
            Some(debate),
        ))
        .await
        .unwrap_err();

    assert!(matches!(error, magi_core::Error::InvalidInput(_)));
    assert_eq!(app.llm.vote_calls(), 0);
}
