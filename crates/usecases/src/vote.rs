//! Final vote stage: three isolated ballots, then the pure verdict.

use magi_adapters::MagiProvider;
use magi_core::{
    FinalVote, PersonaId, Result, VoteBatch, VoteCall, VoteOutcome, VoteRequest, aggregate_votes,
};

use super::App;
use super::budget::{prepare_context, prepare_question, truncate_head};
use super::{MAGI_PEER_BUDGET, validate_debate, validate_openings};

impl<L: MagiProvider> App<L> {
    pub async fn cast_votes(&self, request: VoteRequest) -> Result<VoteBatch> {
        let question = prepare_question(&request.question)?;
        let (context, _) = prepare_context(&request.context);
        validate_openings(&request.openings)?;
        if let Some(debate) = &request.debate {
            validate_debate(debate)?;
        }

        let openings = request.openings;
        let debate = request.debate;

        let futures = PersonaId::ALL.map(|persona| {
            let own_opening = openings.opening(persona).cloned();
            let own_stance = own_opening.as_ref().map(|opening| opening.stance);
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
            let debate_replies = debate
                .as_ref()
                .map(|batch| {
                    batch
                        .successful_replies()
                        .into_iter()
                        .cloned()
                        .map(|mut reply| {
                            reply.text = truncate_head(&reply.text, MAGI_PEER_BUDGET);
                            reply
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let call = VoteCall {
                persona,
                question: question.clone(),
                context: context.clone(),
                own_opening,
                peer_openings,
                debate: debate_replies,
            };

            async move {
                match self.llm.cast_vote(&call).await {
                    Ok(cast) => VoteOutcome::Ok {
                        vote: FinalVote {
                            persona,
                            stance: cast.stance,
                            confidence: cast.confidence,
                            reason: cast.reason,
                            model: cast.model,
                            opening_stance: own_stance,
                            changed_from_opening: own_stance.map(|from| from != cast.stance),
                        },
                    },
                    Err(error) => VoteOutcome::Failed {
                        persona,
                        message: error.to_string(),
                    },
                }
            }
        });

        let [melchior, balthasar, casper] = futures;
        let (melchior, balthasar, casper) = tokio::join!(melchior, balthasar, casper);

        let outcomes = vec![melchior, balthasar, casper];
        let (tally, verdict) = aggregate_votes(&outcomes);

        Ok(VoteBatch {
            outcomes,
            tally,
            verdict,
            prompt_version: VoteBatch::PROMPT_VERSION.into(),
        })
    }
}
