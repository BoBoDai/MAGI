use magi_core::{DebateOutcome, Error, PersonaId, Stance};

use super::support::{FakeProvider, batch_with_failed_opening, debate_request, opening_batch};
use crate::App;

#[tokio::test]
async fn requires_every_opening_to_succeed() {
    let app = App::new(FakeProvider::default());
    let request = debate_request(batch_with_failed_opening(PersonaId::Casper));

    let error = app.run_debate(request, &|_event| {}).await.unwrap_err();

    assert!(matches!(error, Error::InvalidInput(_)));
    assert_eq!(app.llm.debate_calls(), 0);
}

#[tokio::test]
async fn streams_three_debate_replies() {
    let app = App::new(FakeProvider::default());
    let request = debate_request(opening_batch([Stance::Approve; 3]));

    let batch = app.run_debate(request, &|_event| {}).await.unwrap();

    assert_eq!(batch.outcomes.len(), 3);
    assert!(matches!(batch.outcomes[0], DebateOutcome::Ok { .. }));
    assert!(matches!(batch.outcomes[1], DebateOutcome::Ok { .. }));
    assert!(matches!(batch.outcomes[2], DebateOutcome::Ok { .. }));
    assert_eq!(app.llm.debate_calls(), 3);
}

#[tokio::test]
async fn keeps_a_failed_debate_slot() {
    let app = App::new(FakeProvider::default().failing_debate(PersonaId::Casper));
    let request = debate_request(opening_batch([Stance::Approve; 3]));

    let batch = app.run_debate(request, &|_event| {}).await.unwrap();

    assert!(matches!(batch.outcomes[2], DebateOutcome::Failed { .. }));
    assert!(matches!(batch.outcomes[0], DebateOutcome::Ok { .. }));
}

#[tokio::test]
async fn rejects_out_of_order_openings() {
    let app = App::new(FakeProvider::default());
    let mut openings = opening_batch([Stance::Approve; 3]);
    openings.outcomes.swap(0, 1);

    let error = app
        .run_debate(debate_request(openings), &|_event| {})
        .await
        .unwrap_err();

    assert!(matches!(error, Error::InvalidInput(_)));
    assert_eq!(app.llm.debate_calls(), 0);
}
