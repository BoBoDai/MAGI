import type { MagiActions, MagiState, Phase } from '../hooks/useMagiSession';
import { PERSONAS } from '../lib/labels';
import { openingFor, voteFor } from '../lib/sessionSelectors';
import type { PersonaId, Stage } from '../types';
import MagiResponse from './MagiResponse';
import MagiUnit from './MagiUnit';

interface MagiConsoleProps {
  state: MagiState;
  actions: MagiActions;
  demoEnabled: boolean;
  onNewSession: () => void;
  onSelectUnit: (persona: PersonaId) => void;
}

type StepKey = 'opening' | 'debate' | 'vote' | 'verdict';
type StepState = 'done' | 'active' | 'pending' | 'failed';

const STEPS: { key: StepKey; label: string }[] = [
  { key: 'opening', label: 'OPENING' },
  { key: 'debate', label: 'DEBATE' },
  { key: 'vote', label: 'VOTE' },
  { key: 'verdict', label: 'VERDICT' },
];

const STEP_TEXT: Record<StepState, string> = {
  done: 'text-term-green',
  active: 'text-term-amber',
  pending: 'text-term-dim',
  failed: 'text-term-red',
};

function stepState(
  key: StepKey,
  phase: Phase,
  failedStage: Stage | null,
): StepState {
  if (failedStage === key) return 'failed';

  switch (key) {
    case 'opening':
      if (phase === 'idle') return 'pending';
      return phase === 'opening' ? 'active' : 'done';
    case 'debate':
      if (phase === 'opening_ready' || phase === 'debating') return 'active';
      if (phase === 'debate_ready' || phase === 'voting' || phase === 'verdict') {
        return 'done';
      }
      return 'pending';
    case 'vote':
      if (phase === 'opening_ready' || phase === 'debate_ready' || phase === 'voting') {
        return 'active';
      }
      return phase === 'verdict' ? 'done' : 'pending';
    case 'verdict':
      return phase === 'verdict' ? 'active' : 'pending';
  }
}

export default function MagiConsole({
  state,
  actions,
  demoEnabled,
  onNewSession,
  onSelectUnit,
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
  const mode =
    state.phase === 'idle'
      ? 'STANDBY'
      : state.phase.toUpperCase().replace('_', ' ');

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
    'border border-term-amber px-2 py-0.5 text-[9px] tracking-[0.14em] text-term-amber hover:bg-term-amber hover:text-black disabled:cursor-not-allowed disabled:border-term-grid disabled:text-term-dim disabled:hover:bg-transparent';

  return (
    <section className="magi-stage">
      {state.stageError ? (
        <div className="flex flex-wrap items-center gap-2 border border-term-red bg-black px-2 py-1 text-[10px] text-term-red">
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
        <div className="border border-term-amber bg-black px-2 py-1 text-[10px] text-term-amber">
          {state.saveWarning}
        </div>
      ) : null}

      <div className="magi-panel-wrap">
        <div className="magi-panel">
          <div className="magi-connection casper-balthasar" />
          <div className="magi-connection casper-melchior" />
          <div className="magi-connection balthasar-melchior" />

          <div className="magi-header left">
            <hr />
            <hr />
            <span>質 問</span>
            <hr />
            <hr />
          </div>

          <div className="magi-header right">
            <hr />
            <hr />
            <span>解 決</span>
            <hr />
            <hr />
          </div>

          <div className="magi-status">
            <div>CODE:473</div>
            <div>FILE:MAGI_SYS</div>
            <div>MODE:{mode}</div>
            <div>LINK:{demoEnabled ? 'MOCK' : 'LIVE'}</div>
            <div>PRIORITY:AAA</div>
          </div>

          {PERSONAS.map((persona) => (
            <MagiUnit
              key={persona}
              persona={persona}
              status={state.nodes[persona]}
              opening={openingFor(state.openings, persona)}
              vote={voteFor(state.votes, persona)}
              onClick={() => onSelectUnit(persona)}
            />
          ))}

          <MagiResponse state={state} />

          <div className="magi-title">MAGI</div>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2 px-1 py-1">
        <div className="flex items-center gap-1.5 text-[9px] tracking-[0.16em]">
          {STEPS.map((step, index) => {
            const stepStateValue = stepState(step.key, state.phase, state.failedStage);
            return (
              <span key={step.key} className="flex items-center gap-1.5">
                <span className={STEP_TEXT[stepStateValue]}>{step.label}</span>
                {index < STEPS.length - 1 ? (
                  <span className="text-term-grid">›</span>
                ) : null}
              </span>
            );
          })}
        </div>

        <div className="ml-auto flex flex-wrap items-center gap-1.5">
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
            className="border border-term-grid px-2 py-0.5 text-[9px] tracking-[0.14em] text-term-dim hover:border-term-fg hover:text-term-fg disabled:opacity-40"
            onClick={onNewSession}
            disabled={busy}
          >
            NEW SESSION
          </button>
        </div>
      </div>
    </section>
  );
}
