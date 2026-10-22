use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use magi_adapters::MagiProvider;
use magi_core::{
    DebateBatch, DebateCall, DebateOutcome, DebateReply, DebateRequest, Error, Opening,
    OpeningBatch, OpeningCall, OpeningOutcome, OpeningRequest, PersonaId, Result, Stance, VoteCall,
    VoteCast, VoteRequest, decide_openings,
};
use tokio::sync::Barrier;

/// A scriptable provider that records every call and can synchronize the
/// three concurrent opening calls on a barrier.
pub struct FakeProvider {
    pub opening_stances: [Stance; 3],
    pub final_stances: [Stance; 3],
    pub fail_opening: Option<PersonaId>,
    pub fail_debate: Option<PersonaId>,
    pub fail_vote: Option<PersonaId>,
    pub barrier: Option<Arc<Barrier>>,
    pub opening_calls: AtomicUsize,
    pub debate_calls: AtomicUsize,
    pub vote_calls: AtomicUsize,
    pub recorded_openings: Mutex<Vec<OpeningCall>>,
    pub recorded_votes: Mutex<Vec<VoteCall>>,
}

impl Default for FakeProvider {
    fn default() -> Self {
        Self {
            opening_stances: [Stance::Approve; 3],
            final_stances: [Stance::Approve; 3],
            fail_opening: None,
            fail_debate: None,
            fail_vote: None,
            barrier: None,
            opening_calls: AtomicUsize::new(0),
            debate_calls: AtomicUsize::new(0),
            vote_calls: AtomicUsize::new(0),
            recorded_openings: Mutex::new(Vec::new()),
            recorded_votes: Mutex::new(Vec::new()),
        }
    }
}

impl FakeProvider {
    pub fn with_opening_stances(mut self, stances: [Stance; 3]) -> Self {
        self.opening_stances = stances;
        self
    }

    pub fn with_final_stances(mut self, stances: [Stance; 3]) -> Self {
        self.final_stances = stances;
        self
    }

    pub fn failing_opening(mut self, persona: PersonaId) -> Self {
        self.fail_opening = Some(persona);
        self
    }

    pub fn failing_vote(mut self, persona: PersonaId) -> Self {
        self.fail_vote = Some(persona);
        self
    }

    pub fn failing_debate(mut self, persona: PersonaId) -> Self {
        self.fail_debate = Some(persona);
        self
    }

    pub fn with_barrier(mut self, barrier: Arc<Barrier>) -> Self {
        self.barrier = Some(barrier);
        self
    }

    pub fn opening_calls(&self) -> usize {
        self.opening_calls.load(Ordering::SeqCst)
    }

    pub fn debate_calls(&self) -> usize {
        self.debate_calls.load(Ordering::SeqCst)
    }

    pub fn vote_calls(&self) -> usize {
        self.vote_calls.load(Ordering::SeqCst)
    }

    pub fn recorded_openings(&self) -> Vec<OpeningCall> {
        self.recorded_openings.lock().unwrap().clone()
    }

    pub fn recorded_votes(&self) -> Vec<VoteCall> {
        self.recorded_votes.lock().unwrap().clone()
    }
}

pub fn persona_index(persona: PersonaId) -> usize {
    match persona {
        PersonaId::Melchior => 0,
        PersonaId::Balthasar => 1,
        PersonaId::Casper => 2,
    }
}

impl MagiProvider for FakeProvider {
    async fn opening_stream(
        &self,
        call: &OpeningCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<Opening> {
        self.opening_calls.fetch_add(1, Ordering::SeqCst);
        self.recorded_openings.lock().unwrap().push(call.clone());
        on_delta(format!("{} opening", call.persona.as_str()));

        if let Some(barrier) = &self.barrier {
            barrier.wait().await;
        }

        if self.fail_opening == Some(call.persona) {
            return Err(Error::Llm("opening failed".into()));
        }

        Ok(Opening {
            persona: call.persona,
            stance: self.opening_stances[persona_index(call.persona)],
            confidence: 70,
            summary: format!("{} summary", call.persona.as_str()),
            key_points: vec!["point".into()],
            risks: vec!["risk".into()],
            model: "fake".into(),
        })
    }

    async fn debate_stream(
        &self,
        call: &DebateCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<DebateReply> {
        self.debate_calls.fetch_add(1, Ordering::SeqCst);
        on_delta(format!("{} debate", call.persona.as_str()));

        if self.fail_debate == Some(call.persona) {
            return Err(Error::Llm("debate failed".into()));
        }

        Ok(DebateReply {
            persona: call.persona,
            text: format!("{} debate reply", call.persona.as_str()),
            model: "fake".into(),
        })
    }

    async fn cast_vote(&self, call: &VoteCall) -> Result<VoteCast> {
        self.vote_calls.fetch_add(1, Ordering::SeqCst);
        self.recorded_votes.lock().unwrap().push(call.clone());

        if self.fail_vote == Some(call.persona) {
            return Err(Error::Llm("vote failed".into()));
        }

        Ok(VoteCast {
            persona: call.persona,
            stance: self.final_stances[persona_index(call.persona)],
            confidence: 80,
            reason: "reason".into(),
            model: "fake".into(),
        })
    }
}

pub fn opening_request(question: &str) -> OpeningRequest {
    OpeningRequest {
        run_id: "run-1".into(),
        question: question.into(),
        context: String::new(),
    }
}

pub fn opening_batch(stances: [Stance; 3]) -> OpeningBatch {
    let outcomes: Vec<_> = PersonaId::ALL
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
                model: "fake".into(),
            },
        })
        .collect();
    let decision = decide_openings(&outcomes);

    OpeningBatch {
        outcomes,
        prompt_version: OpeningBatch::PROMPT_VERSION.into(),
        decision,
    }
}

pub fn batch_with_failed_opening(failed: PersonaId) -> OpeningBatch {
    let mut batch = opening_batch([Stance::Approve; 3]);
    let index = persona_index(failed);
    batch.outcomes[index] = OpeningOutcome::Failed {
        persona: failed,
        message: "opening failed".into(),
    };
    batch.decision = decide_openings(&batch.outcomes);
    batch
}

pub fn debate_request(openings: OpeningBatch) -> DebateRequest {
    DebateRequest {
        run_id: "run-1".into(),
        question: "Should we ship?".into(),
        context: String::new(),
        openings,
    }
}

pub fn vote_request(openings: OpeningBatch, debate: Option<DebateBatch>) -> VoteRequest {
    VoteRequest {
        question: "Should we ship?".into(),
        context: String::new(),
        openings,
        debate,
    }
}

pub fn debate_batch() -> DebateBatch {
    DebateBatch {
        outcomes: PersonaId::ALL
            .into_iter()
            .map(|persona| DebateOutcome::Ok {
                reply: DebateReply {
                    persona,
                    text: format!("{} debate", persona.as_str()),
                    model: "fake".into(),
                },
            })
            .collect(),
        prompt_version: DebateBatch::PROMPT_VERSION.into(),
    }
}
