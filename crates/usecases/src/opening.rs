//! Opening stage: three concurrent, independent statements.

use magi_adapters::MagiProvider;
use magi_core::{
    MagiStreamEvent, OpeningBatch, OpeningCall, OpeningOutcome, OpeningRequest, PersonaId, Result,
    Stage, decide_openings,
};

use super::App;
use super::budget::{prepare_context, prepare_question};

impl<L: MagiProvider> App<L> {
    pub async fn run_opening<E>(&self, request: OpeningRequest, emit: &E) -> Result<OpeningBatch>
    where
        E: Fn(MagiStreamEvent) + Send + Sync,
    {
        let question = prepare_question(&request.question)?;
        let (context, _) = prepare_context(&request.context);
        let run_id = request.run_id;
        let stage = Stage::Opening;

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
            let call = OpeningCall {
                persona,
                question: question.clone(),
                context: context.clone(),
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

                match self.llm.opening_stream(&call, &mut on_delta).await {
                    Ok(opening) => {
                        emit(MagiStreamEvent::PersonaCompleted {
                            run_id: run_id.clone(),
                            stage,
                            persona,
                        });
                        OpeningOutcome::Ok { opening }
                    }
                    Err(error) => {
                        let message = error.to_string();
                        emit(MagiStreamEvent::PersonaFailed {
                            run_id: run_id.clone(),
                            stage,
                            persona,
                            message: message.clone(),
                        });
                        OpeningOutcome::Failed { persona, message }
                    }
                }
            }
        });

        let [melchior, balthasar, casper] = futures;
        let (melchior, balthasar, casper) = tokio::join!(melchior, balthasar, casper);

        emit(MagiStreamEvent::StageCompleted { run_id, stage });

        let outcomes = vec![melchior, balthasar, casper];
        let decision = decide_openings(&outcomes);

        Ok(OpeningBatch {
            outcomes,
            prompt_version: OpeningBatch::PROMPT_VERSION.into(),
            decision,
        })
    }
}
