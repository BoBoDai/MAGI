use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use magi_core::{
    DebateBatch, DebateOutcome, DebateReply, FinalVote, MagiVerdict, Opening, OpeningBatch,
    OpeningOutcome, PersonaId, SessionDraft, SessionSource, SessionStatus, Stance, VoteBatch,
    VoteOutcome, VoteTally,
};

use super::SessionStore;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("magi-sessions-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

fn openings() -> OpeningBatch {
    OpeningBatch {
        outcomes: PersonaId::ALL
            .into_iter()
            .map(|persona| OpeningOutcome::Ok {
                opening: Opening {
                    persona,
                    stance: Stance::Approve,
                    confidence: 70,
                    summary: "summary".into(),
                    key_points: vec!["point".into()],
                    risks: vec!["risk".into()],
                    model: "test".into(),
                },
            })
            .collect(),
        prompt_version: OpeningBatch::PROMPT_VERSION.into(),
        decision: magi_core::DecisionStatus::Yes,
    }
}

fn vote_batch(stances: [Stance; 3]) -> VoteBatch {
    VoteBatch {
        outcomes: PersonaId::ALL
            .into_iter()
            .zip(stances)
            .map(|(persona, stance)| VoteOutcome::Ok {
                vote: FinalVote {
                    persona,
                    stance,
                    confidence: 80,
                    reason: "reason".into(),
                    model: "test".into(),
                    opening_stance: None,
                    changed_from_opening: None,
                },
            })
            .collect(),
        tally: VoteTally::default(),
        verdict: MagiVerdict::Incomplete,
        prompt_version: VoteBatch::PROMPT_VERSION.into(),
    }
}

fn draft(id: &str) -> SessionDraft {
    SessionDraft {
        id: id.into(),
        question: "Should we ship?".into(),
        context: String::new(),
        openings: Some(openings()),
        debate: None,
        vote: None,
    }
}

fn debate() -> DebateBatch {
    DebateBatch {
        outcomes: PersonaId::ALL
            .into_iter()
            .map(|persona| DebateOutcome::Ok {
                reply: DebateReply {
                    persona,
                    text: "debate".into(),
                    model: "test".into(),
                },
            })
            .collect(),
        prompt_version: DebateBatch::PROMPT_VERSION.into(),
    }
}

#[test]
fn saves_lists_and_deletes() {
    let dir = temp_dir("crud");
    let store = SessionStore::load(dir.clone());

    store.save(draft("s1"), SessionSource::Live).unwrap();
    sleep(Duration::from_millis(2));
    store.save(draft("s2"), SessionSource::Mock).unwrap();

    let list = store.list();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, "s2");
    assert_eq!(list[0].status, SessionStatus::Opening);

    let record = store.get("s2").unwrap();
    assert_eq!(record.source, SessionSource::Mock);

    store.delete("s1").unwrap();
    assert!(store.get("s1").is_none());
    assert_eq!(store.list().len(), 1);

    let reloaded = SessionStore::load(dir.clone());
    assert_eq!(reloaded.list().len(), 1);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn recomputes_verdict_and_changes_before_persisting() {
    let dir = temp_dir("recompute");
    let store = SessionStore::load(dir.clone());

    let mut draft = draft("s1");
    draft.vote = Some(vote_batch([Stance::Approve; 3]));
    let summary = store.save(draft, SessionSource::Live).unwrap();

    assert_eq!(
        summary.verdict,
        Some(MagiVerdict::Unanimous {
            stance: Stance::Approve
        })
    );

    let record = store.get("s1").unwrap();
    let vote = record.vote.unwrap();
    assert_eq!(
        vote.verdict,
        MagiVerdict::Unanimous {
            stance: Stance::Approve
        }
    );
    assert_eq!(vote.tally.approve, 3);
    assert_eq!(
        vote.vote(PersonaId::Melchior).unwrap().changed_from_opening,
        Some(false)
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn upsert_preserves_created_at() {
    let dir = temp_dir("upsert");
    let store = SessionStore::load(dir.clone());

    let first = store.save(draft("s1"), SessionSource::Live).unwrap();
    sleep(Duration::from_millis(3));

    let mut updated = draft("s1");
    updated.debate = Some(debate());
    let second = store.save(updated, SessionSource::Live).unwrap();

    assert_eq!(first.created_at, second.created_at);
    assert_ne!(first.updated_at, second.updated_at);
    assert_eq!(store.list().len(), 1);
    assert_eq!(store.list()[0].status, SessionStatus::Debate);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn caps_history_at_the_limit() {
    let dir = temp_dir("cap");
    let store = SessionStore::load(dir.clone());

    for index in 0..105 {
        store
            .save(draft(&format!("s{index:03}")), SessionSource::Live)
            .unwrap();
    }

    let list = store.list();
    assert_eq!(list.len(), magi_core::MAX_STORED_SESSIONS);
    assert_eq!(list[0].id, "s104");
    assert!(list.iter().all(|item| item.id != "s004"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn backs_up_a_corrupt_history_file() {
    let dir = temp_dir("corrupt");
    fs::write(dir.join("sessions.json"), "{not json").unwrap();

    let store = SessionStore::load(dir.clone());
    assert!(store.list().is_empty());

    let backup = fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("sessions.corrupt-")
        });
    assert!(backup.is_some(), "corrupt file should be backed up");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn refuses_to_overwrite_a_newer_schema() {
    let dir = temp_dir("future");
    let path = dir.join("sessions.json");
    fs::write(&path, r#"{"schema_version":99,"sessions":[]}"#).unwrap();

    let store = SessionStore::load(dir.clone());
    assert!(store.save(draft("s1"), SessionSource::Live).is_err());

    let raw = fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed["schema_version"], 99);

    let _ = fs::remove_dir_all(dir);
}
