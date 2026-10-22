//! Debate stage: one round, parallel, no visibility of peer debate replies.

use magi_adapters::MagiProvider;
use magi_core::{
    DebateBatch, DebateCall, DebateOutcome, DebateRequest, Error, MagiStreamEvent, PersonaId,
    Result, Stage,
};

use super::App;
use super::budget::{prepare_context, prepare_question, truncate_head};
use super::{MAGI_PEER_BUDGET, validate_openings};

impl<L: MagiProvider> App<L> {
    pub async fn run_debate<E>(&self, request: DebateRequest, emit: &E) -> Result<DebateBatch>
    where
        E: Fn(MagiStreamEvent) + Send + Sync,
    {
        let question = prepare_question(&request.question)?;
        let (context, _) = prepare_context(&request.context);
        validate_openings(&request.openings)?;

        if !request.openings.all_succeeded() {
            return Err(Error::InvalidInput(
                "交叉质询需要三个脑的开场全部成功".into(),
            ));
        }

        let run_id = request.run_id;
        let stage = Stage::Debate;
        let openings = request.openings;

        emit(MagiStreamEvent::StageStarted {
            run_id: run_id.clone(),
            stage,
            personas: PersonaId::ALL.to_vec(),
        });
        for persona in PersonaId::ALL {
            emit(MagiStreamEvent::PersonaStarted {
                run_id: run_id.clone(),
                stage,
                persona,
            });
        }

        let futures = PersonaId::ALL.map(|persona| {
            let own_opening = openings
                .opening(persona)
                .cloned()
                .expect("all openings validated as successful");
            let peer_openings = openings
                .successful_openings()
                .into_iter()
                .filter(|opening| opening.persona != persona)
                .cloned()
                .map(|mut opening| {
                    opening.summary = truncate_head(&opening.summary, MAGI_PEER_BUDGET);
                    opening
                })
                .collect::<Vec<_>>();
            let call = DebateCall {
                persona,
                question: question.clone(),
                context: context.clone(),
                own_opening,
                peer_openings,
            };
            let run_id = run_id.clone();

            async move {
                let mut on_delta = |delta: String| {
                    emit(MagiStreamEvent::Delta {
                        run_id: run_id.clone(),
                        stage,
                        persona,
                        text: delta,
                    });
                };

                match self.llm.debate_stream(&call, &mut on_delta).await {
                    Ok(reply) => {
                        emit(MagiStreamEvent::PersonaCompleted {
                            run_id: run_id.clone(),
                            stage,
                            persona,
                        });
                        DebateOutcome::Ok { reply }
                    }
                    Err(error) => {
                        let message = error.to_string();
                        emit(MagiStreamEvent::PersonaFailed {
                            run_id: run_id.clone(),
                            stage,
                            persona,
                            message: message.clone(),
                        });
                        DebateOutcome::Failed { persona, message }
                    }
                }
            }
        });

        let [melchior, balthasar, casper] = futures;
        let (melchior, balthasar, casper) = tokio::join!(melchior, balthasar, casper);

        emit(MagiStreamEvent::StageCompleted { run_id, stage });

        Ok(DebateBatch {
            outcomes: vec![melchior, balthasar, casper],
            prompt_version: DebateBatch::PROMPT_VERSION.into(),
        })
    }
}
