//! # magi-core
//!
//! Pure domain layer. No HTTP, no filesystem, no Tauri types.
//!
//! The three MAGI brains are modeled as [`persona::PersonaId`]. Protocol
//! stages, ballots, stream events, and session records all live here so the
//! transport and storage layers cannot drift from the decision policy.

pub mod debate;
pub mod decision;
pub mod error;
pub mod opening;
pub mod persona;
pub mod session;
pub mod stage;
pub mod verdict;
pub mod vote;

pub use debate::{DebateBatch, DebateCall, DebateOutcome, DebateReply, DebateRequest};
pub use decision::{DecisionStatus, decide_openings};
pub use error::{Error, Result};
pub use opening::{Opening, OpeningBatch, OpeningCall, OpeningOutcome, OpeningRequest};
pub use persona::PersonaId;
pub use session::{
    MAX_STORED_SESSIONS, SESSION_SCHEMA_VERSION, SessionDraft, SessionRecord, SessionSource,
    SessionStatus, SessionSummary,
};
pub use stage::{MagiStreamEvent, Stage, Stance};
pub use verdict::aggregate_votes;
pub use vote::{
    FinalVote, MagiVerdict, VoteBatch, VoteCall, VoteCast, VoteOutcome, VoteRequest, VoteTally,
};
