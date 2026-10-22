import type { Phase } from '../hooks/useMagiSession';
import type { Stage } from '../types';

interface StageRailProps {
  phase: Phase;
  failedStage: Stage | null;
}

type Step = { key: 'opening' | 'debate' | 'vote' | 'verdict'; label: string };

const STEPS: Step[] = [
  { key: 'opening', label: 'OPENING' },
  { key: 'debate', label: 'DEBATE' },
  { key: 'vote', label: 'VOTE' },
  { key: 'verdict', label: 'VERDICT' },
];

type StepState = 'done' | 'active' | 'pending' | 'failed';

function stepState(
  key: Step['key'],
  phase: Phase,
  failedStage: Stage | null,
): StepState {
  if (failedStage === key) return 'failed';

  switch (key) {
    case 'opening':
      if (phase === 'idle') return 'pending';
      return phase === 'opening' ? 'active' : 'done';
    case 'debate':
      if (phase === 'opening_ready') return 'active';
      if (phase === 'debating') return 'active';
      if (phase === 'debate_ready' || phase === 'voting' || phase === 'verdict') {
        return 'done';
      }
      return 'pending';
    case 'vote':
      if (phase === 'opening_ready' || phase === 'debate_ready') return 'active';
      if (phase === 'voting') return 'active';
      if (phase === 'verdict') return 'done';
      return 'pending';
    case 'verdict':
      return phase === 'verdict' ? 'active' : 'pending';
  }
}

const STATE_CLASS: Record<StepState, string> = {
  done: 'border-term-green text-term-green',
  active: 'border-term-amber bg-term-amber text-term-bg',
  pending: 'border-term-grid text-term-dim',
  failed: 'border-term-red text-term-red',
};

const MARK: Record<StepState, string> = {
  done: '✓',
  active: '▸',
  pending: '·',
  failed: '!',
};

export default function StageRail({ phase, failedStage }: StageRailProps) {
  return (
    <div className="flex flex-wrap items-center gap-1.5 border-b border-term-grid px-4 py-1.5">
      {STEPS.map((step) => {
        const state = stepState(step.key, phase, failedStage);
        return (
          <span
            key={step.key}
            className={`flex items-center gap-1 border px-2 py-0.5 text-[9px] tracking-[0.18em] ${STATE_CLASS[state]}`}
          >
            <span>{MARK[state]}</span>
            {step.label}
          </span>
        );
      })}
      <span className="ml-auto text-[9px] tracking-[0.16em] text-term-dim">
        {phase === 'idle' ? 'AWAITING QUERY' : phase.toUpperCase().replace('_', ' ')}
      </span>
    </div>
  );
}
