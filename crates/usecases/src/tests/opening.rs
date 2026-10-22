use std::sync::{Arc, Mutex};
use std::time::Duration;

use magi_core::{MagiStreamEvent, OpeningOutcome, PersonaId, Stage, Stance};
use tokio::sync::Barrier;

use super::support::{FakeProvider, opening_request};
use crate::{App, MAGI_CONTEXT_BUDGET, MAGI_QUESTION_BUDGET};

#[tokio::test]
async fn runs_three_openings_in_persona_order() {
    let app = App::new(FakeProvider::default().with_opening_stances([
        Stance::Approve,
        Stance::Reject,
        Stance::Abstain,
    ]));

    let batch = app
        .run_opening(opening_request("Should we ship?"), &|_event| {})
        .await
        .unwrap();

    assert_eq!(batch.outcomes.len(), 3);
    for (outcome, persona) in batch.outcomes.iter().zip(PersonaId::ALL) {
        assert_eq!(outcome.persona(), persona);
    }
    assert_eq!(app.llm.opening_calls(), 3);
}

#[tokio::test]
async fn three_openings_run_concurrently() {
    let barrier = Arc::new(Barrier::new(3));
    let app = App::new(FakeProvider::default().with_barrier(barrier));

    let result = tokio::time::timeout(
        Duration::from_secs(2),
        app.run_opening(opening_request("q"), &|_event| {}),
    )
    .await;

    assert!(result.is_ok(), "opening calls did not run concurrently");
    assert!(result.unwrap().is_ok());
}

#[tokio::test]
async fn emits_tagged_deltas_and_completion_events() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    let emit = move |event: MagiStreamEvent| sink.lock().unwrap().push(event);

    let app = App::new(FakeProvider::default());
    app.run_opening(opening_request("q"), &emit).await.unwrap();

    let events = events.lock().unwrap();
    let deltas: Vec<_> = events
        .iter()
        .filter(|event| matches!(event, MagiStreamEvent::Delta { .. }))
        .collect();
    let completed: Vec<_> = events
        .iter()
        .filter(|event| matches!(event, MagiStreamEvent::PersonaCompleted { .. }))
        .collect();

    assert_eq!(deltas.len(), 3);
    assert_eq!(completed.len(), 3);
    assert!(events.iter().any(|event| matches!(
        event,
        MagiStreamEvent::StageStarted {
            stage: Stage::Opening,
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        MagiStreamEvent::StageCompleted {
            stage: Stage::Opening,
            ..
        }
    )));
}

#[tokio::test]
async fn keeps_a_failed_slot_instead_of_dropping_it() {
    let app = App::new(FakeProvider::default().failing_opening(PersonaId::Balthasar));

    let batch = app
        .run_opening(opening_request("q"), &|_event| {})
        .await
        .unwrap();

    assert!(matches!(batch.outcomes[0], OpeningOutcome::Ok { .. }));
    assert!(matches!(batch.outcomes[1], OpeningOutcome::Failed { .. }));
    assert!(matches!(batch.outcomes[2], OpeningOutcome::Ok { .. }));
    assert_eq!(app.llm.opening_calls(), 3);
}

#[tokio::test]
async fn rejects_empty_and_over_budget_questions() {
    let app = App::new(FakeProvider::default());
    assert!(
        app.run_opening(opening_request("  "), &|_event| {})
            .await
            .is_err()
    );

    let long = "问".repeat(MAGI_QUESTION_BUDGET + 1);
    assert!(
        app.run_opening(opening_request(&long), &|_event| {})
            .await
            .is_err()
    );
    assert_eq!(app.llm.opening_calls(), 0);
}

#[tokio::test]
async fn truncates_long_context_before_prompting() {
    let app = App::new(FakeProvider::default());
    let mut request = opening_request("q");
    request.context = "材".repeat(50_000);

    app.run_opening(request, &|_event| {}).await.unwrap();

    let calls = app.llm.recorded_openings();
    assert_eq!(calls.len(), 3);
    let context = &calls[0].context;
    assert!(context.chars().count() <= MAGI_CONTEXT_BUDGET + 100);
    assert!(context.contains("中间内容已省略"));
}
