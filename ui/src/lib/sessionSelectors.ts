import type {
  DebateBatch,
  DebateReply,
  FinalVote,
  Opening,
  OpeningBatch,
  PersonaId,
  VoteBatch,
} from '../types';

export function openingFor(
  batch: OpeningBatch | null,
  persona: PersonaId,
): Opening | null {
  const outcome = batch?.outcomes.find((item) =>
    item.status === 'ok' ? item.opening.persona === persona : item.persona === persona,
  );
  return outcome?.status === 'ok' ? outcome.opening : null;
}

export function debateFor(
  batch: DebateBatch | null,
  persona: PersonaId,
): DebateReply | null {
  const outcome = batch?.outcomes.find((item) =>
    item.status === 'ok' ? item.reply.persona === persona : item.persona === persona,
  );
  return outcome?.status === 'ok' ? outcome.reply : null;
}

export function voteFor(
  batch: VoteBatch | null,
  persona: PersonaId,
): FinalVote | null {
  const outcome = batch?.outcomes.find((item) =>
    item.status === 'ok' ? item.vote.persona === persona : item.persona === persona,
  );
  return outcome?.status === 'ok' ? outcome.vote : null;
}
