import type { MagiActions, MagiState } from '../hooks/useMagiSession';
import { PERSONAS } from '../lib/labels';
import type { DebateBatch, OpeningBatch, PersonaId, VoteBatch } from '../types';
import StageRail from './StageRail';
import UnitColumn from './UnitColumn';
import VerdictPanel from './VerdictPanel';

interface MagiConsoleProps {
  state: MagiState;
  actions: MagiActions;
  onNewSession: () => void;
}

function openingFor(batch: OpeningBatch | null, persona: PersonaId) {
  const outcome = batch?.outcomes.find((item) =>
    item.status === 'ok' ? item.opening.persona === persona : item.persona === persona,
  );
  return outcome?.status === 'ok' ? outcome.opening : null;
}

function debateFor(batch: DebateBatch | null, persona: PersonaId) {
  const outcome = batch?.outcomes.find((item) =>
    item.status === 'ok' ? item.reply.persona === persona : item.persona === persona,
  );
  return outcome?.status === 'ok' ? outcome.reply : null;
}

function voteFor(batch: VoteBatch | null, persona: PersonaId) {
  const outcome = batch?.outcomes.find((item) =>
    item.status === 'ok' ? item.vote.persona === persona : item.persona === persona,
  );
  return outcome?.status === 'ok' ? outcome.vote : null;
}

export default function MagiConsole({
  state,
  actions,
  onNewSession,
}: MagiConsoleProps) {
  const busy =
    state.phase === 'opening' ||
    state.phase === 'debating' ||
    state.phase === 'voting';
  const allOpeningsOk =
    state.openings?.outcomes.every((outcome) => outcome.status === 'ok') ?? false;
  const canDebate =
    state.phase === 'opening_ready' && allOpeningsOk && !state.readOnly && !busy;
  const canVote =
    (state.phase === 'opening_ready' || state.phase === 'debate_ready') &&
    !state.readOnly &&
    !busy;
  const canRetry = Boolean(state.failedStage) && !busy && !state.readOnly;

  const retry = () => {
    if (state.failedStage === 'opening') {
      void actions.start(state.question, state.context);
    } else if (state.failedStage === 'debate') {
      void actions.debate();
    } else if (state.failedStage === 'vote') {
      void actions.vote();
    }
  };

  const actionClass =
    'border border-term-amber px-2.5 py-1 text-[9px] tracking-[0.16em] text-term-amber hover:bg-term-amber hover:text-term-bg disabled:cursor-not-allowed disabled:border-term-grid disabled:text-term-dim disabled:hover:bg-transparent';

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <StageRail phase={state.phase} failedStage={state.failedStage} />

      {state.stageError ? (
        <div className="flex flex-wrap items-center gap-2 border-b border-term-red bg-term-red/10 px-4 py-1.5 text-[10.5px] text-term-red">
          <span>{state.stageError}</span>
          {canRetry ? (
            <button
              type="button"
              className="border border-term-red px-2 py-0.5 text-[9px] tracking-[0.14em]"
              onClick={retry}
            >
              RETRY {state.failedStage?.toUpperCase()}
            </button>
          ) : null}
        </div>
      ) : null}

      {state.saveWarning ? (
        <div className="border-b border-term-amber bg-term-amber/10 px-4 py-1.5 text-[10px] text-term-amber">
          {state.saveWarning}
        </div>
      ) : null}

      <div className="grid min-h-0 flex-1 grid-cols-3 overflow-hidden border-b border-term-grid max-[900px]:grid-cols-1 max-[900px]:overflow-y-auto">
        {PERSONAS.map((persona, index) => (
          <UnitColumn
            key={persona}
            index={index}
            persona={persona}
            status={state.nodes[persona]}
            stream={state.streaming[persona]}
            opening={openingFor(state.openings, persona)}
            debate={debateFor(state.debate, persona)}
            vote={voteFor(state.votes, persona)}
            error={state.nodeErrors[persona]}
            phase={state.phase}
            readOnly={state.readOnly}
          />
        ))}
      </div>

      <VerdictPanel votes={state.votes} phase={state.phase} />

      <div className="flex flex-wrap items-center gap-2 px-4 py-2">
        <button
          type="button"
          className={actionClass}
          disabled={!canDebate}
          onClick={() => void actions.debate()}
        >
          CROSS-EXAMINATION
        </button>
        <button
          type="button"
          className={actionClass}
          disabled={!canVote}
          onClick={() => void actions.vote()}
        >
          FINAL VOTE
        </button>
        <button
          type="button"
          className="border border-term-grid px-2.5 py-1 text-[9px] tracking-[0.16em] text-term-dim hover:border-term-fg hover:text-term-fg disabled:opacity-40"
          onClick={onNewSession}
          disabled={busy}
        >
          NEW SESSION
        </button>
        <span className="ml-auto text-[9px] tracking-[0.16em] text-term-dim">
          {state.readOnly ? 'ARCHIVE VIEW' : 'LIVE CONSOLE'}
        </span>
      </div>
    </section>
  );
}
