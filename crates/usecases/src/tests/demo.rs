//! End-to-end protocol test with the deterministic demo provider.

use magi_adapters::DemoProvider;
use magi_core::{OpeningRequest, PersonaId};

use crate::App;

#[tokio::test]
async fn runs_the_full_protocol_with_demo_data() {
    let app = App::new(DemoProvider::new());
    let question = "Should we ship the release on Friday?";
    let emit = |_event| {};

    let openings = app
        .run_opening(
            OpeningRequest {
                run_id: "demo-1".into(),
                question: question.into(),
                context: String::new(),
            },
            &emit,
        )
        .await
        .unwrap();

    assert_eq!(openings.outcomes.len(), 3);
    assert!(
        openings
            .outcomes
            .iter()
            .all(|outcome| matches!(outcome, magi_core::OpeningOutcome::Ok { .. }))
    );
    assert!(
        openings
            .successful_openings()
            .iter()
            .all(|opening| opening.model == "mock")
    );

    let debate = app
        .run_debate(
            magi_core::DebateRequest {
                run_id: "demo-2".into(),
                question: question.into(),
                context: String::new(),
                openings: openings.clone(),
            },
            &emit,
        )
        .await
        .unwrap();

    assert!(
        debate
            .outcomes
            .iter()
            .all(|outcome| matches!(outcome, magi_core::DebateOutcome::Ok { .. }))
    );

    let votes = app
        .cast_votes(magi_core::VoteRequest {
            question: question.into(),
            context: String::new(),
            openings,
            debate: Some(debate),
        })
        .await
        .unwrap();

    assert_eq!(votes.outcomes.len(), 3);
    assert!(
        votes
            .outcomes
            .iter()
            .all(|outcome| matches!(outcome, magi_core::VoteOutcome::Ok { .. }))
    );
    assert_ne!(votes.verdict, magi_core::MagiVerdict::Incomplete);
    assert_eq!(votes.vote(PersonaId::Melchior).unwrap().model, "mock");
}
