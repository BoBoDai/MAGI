import type { Phase } from '../hooks/useMagiSession';
import {
  PERSONA_META,
  STANCE_LABEL,
  STANCE_TEXT,
  verdictExplanation,
  verdictTitle,
} from '../lib/labels';
import type { PersonaId, VoteBatch } from '../types';

interface VerdictPanelProps {
  votes: VoteBatch | null;
  phase: Phase;
}

const TALLY_LABELS: { key: 'approve' | 'reject' | 'abstain' | 'failed'; label: string }[] = [
  { key: 'approve', label: 'APPROVE' },
  { key: 'reject', label: 'REJECT' },
  { key: 'abstain', label: 'ABSTAIN' },
  { key: 'failed', label: 'FAILED' },
];

function pendingExplanation(phase: Phase): string {
  switch (phase) {
    case 'opening':
      return '三脑正在形成各自的开场判断，彼此不可见。';
    case 'debating':
      return '交叉质询进行中；本轮结束后可进入独立终审。';
    case 'voting':
      return '三脑正在独立投票，终审票面互不可见。';
    default:
      return '等待三脑完成各自的判断。';
  }
}

function findVote(batch: VoteBatch, persona: PersonaId) {
  return batch.outcomes.find((outcome) =>
    outcome.status === 'ok'
      ? outcome.vote.persona === persona
      : outcome.persona === persona,
  );
}

export default function VerdictPanel({ votes, phase }: VerdictPanelProps) {
  if (!votes) {
    return (
      <section className="border-b border-term-grid bg-term-panel px-4 py-2">
        <div className="flex flex-wrap items-center gap-3">
          <span className="text-[11px] tracking-[0.22em] text-term-dim">
            VERDICT // PENDING
          </span>
          <span className="text-[10px] tracking-[0.1em] text-term-dim">
            {pendingExplanation(phase)}
          </span>
        </div>
      </section>
    );
  }

  const changed = votes.outcomes.filter(
    (outcome) => outcome.status === 'ok' && outcome.vote.changed_from_opening,
  ).length;
  const verdictClass =
    votes.verdict.kind === 'unanimous' || votes.verdict.kind === 'majority'
      ? STANCE_TEXT[votes.verdict.stance]
      : votes.verdict.kind === 'split'
        ? 'text-term-amber'
        : 'text-term-red';

  return (
    <section className="border-b border-term-grid bg-term-panel px-4 py-2">
      <div className="flex flex-wrap items-baseline gap-3">
        <span className={`text-[18px] font-semibold tracking-[0.16em] ${verdictClass}`}>
          {verdictTitle(votes.verdict)}
        </span>
        <span className="text-[10.5px] text-term-dim">
          {verdictExplanation(votes.verdict)}
        </span>
        <div className="ml-auto flex flex-wrap items-center gap-1.5">
          {TALLY_LABELS.map(({ key, label }) => (
            <span
              key={key}
              className="border border-term-grid px-1.5 py-0.5 text-[8.5px] tracking-[0.14em] text-term-dim"
            >
              {label} {votes.tally[key]}
            </span>
          ))}
          <span className="border border-term-grid px-1.5 py-0.5 text-[8.5px] tracking-[0.14em] text-term-amber">
            CHANGED {changed}
          </span>
        </div>
      </div>

      <div className="mt-1.5 grid grid-cols-3 gap-2 max-[900px]:grid-cols-1">
        {(['melchior', 'balthasar', 'casper'] as PersonaId[]).map((persona) => {
          const outcome = findVote(votes, persona);
          const meta = PERSONA_META[persona];
          if (!outcome) {
            return (
              <div key={persona} className="border border-term-grid px-2 py-1">
                <div className="text-[9px] tracking-[0.14em] text-term-dim">
                  {meta.unit} // NO RECORD
                </div>
              </div>
            );
          }

          if (outcome.status === 'failed') {
            return (
              <div key={persona} className="border border-term-red px-2 py-1">
                <div className="text-[9px] tracking-[0.14em] text-term-red">
                  {meta.unit} // FAILED
                </div>
                <div className="mt-0.5 text-[10px] text-term-red">{outcome.message}</div>
              </div>
            );
          }

          return (
            <div key={persona} className="border border-term-grid px-2 py-1">
              <div className="flex items-center gap-2">
                <span className="text-[9px] tracking-[0.14em] text-term-dim">
                  {meta.unit}
                </span>
                <span
                  className={`text-[9px] tracking-[0.14em] ${STANCE_TEXT[outcome.vote.stance]}`}
                >
                  {STANCE_LABEL[outcome.vote.stance]} · {outcome.vote.confidence}
                </span>
              </div>
              <div className="mt-0.5 line-clamp-2 text-[10px] leading-snug text-term-fg">
                {outcome.vote.reason}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
