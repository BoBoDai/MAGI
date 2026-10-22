import type { NodeStatus, Phase } from '../hooks/useMagiSession';
import { PERSONA_META, STANCE_LABEL, STANCE_LED } from '../lib/labels';
import type { DebateReply, FinalVote, Opening, PersonaId } from '../types';
import UnitDetail from './UnitDetail';

interface UnitColumnProps {
  index: number;
  persona: PersonaId;
  status: NodeStatus;
  stream: string;
  opening: Opening | null;
  debate: DebateReply | null;
  vote: FinalVote | null;
  error: string | null;
  phase: Phase;
  readOnly: boolean;
}

export default function UnitColumn({
  index,
  persona,
  status,
  stream,
  opening,
  debate,
  vote,
  error,
  phase,
  readOnly,
}: UnitColumnProps) {
  const meta = PERSONA_META[persona];
  const stance = vote?.stance ?? opening?.stance ?? null;
  const confidence = vote?.confidence ?? opening?.confidence ?? null;
  const failed = Boolean(error) || status === 'failed';

  const ledClass = failed
    ? 'bg-term-red'
    : status === 'thinking'
      ? 'bg-term-amber animate-pulse'
      : stance
        ? STANCE_LED[stance]
        : status === 'done'
          ? 'bg-term-green'
          : 'bg-term-grid';

  const statusLabel = failed
    ? 'FAILED'
    : status === 'thinking'
      ? 'PROCESSING'
      : stance
        ? STANCE_LABEL[stance]
        : status === 'done'
          ? 'COMPLETE'
          : 'STANDBY';

  return (
    <div className="flex h-full min-h-0 flex-col border-r border-term-grid last:border-r-0 max-[900px]:border-b max-[900px]:border-r-0">
      <div className="flex items-center gap-2 border-b border-term-grid px-3 py-1.5">
        <span className={`h-1.5 w-1.5 ${ledClass}`} />
        <span className="text-[9px] tracking-[0.16em] text-term-dim">
          UNIT 0{index + 1}
        </span>
        <span className="text-[10px] tracking-[0.14em] text-term-amber">
          {meta.unit}
        </span>
        <span className="ml-auto text-[9px] tracking-[0.1em] text-term-dim">
          {statusLabel}
        </span>
        <span className="w-8 text-right text-lg leading-none">
          {confidence ?? '—'}
        </span>
      </div>

      <div className="border-b border-term-grid px-3 py-1 text-[9.5px] tracking-[0.12em] text-term-dim">
        {meta.label} · {meta.lens}
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-3 py-2">
        <UnitDetail
          status={status}
          stream={stream}
          opening={opening}
          debate={debate}
          vote={vote}
          error={error}
          phase={phase}
          readOnly={readOnly}
        />
      </div>
    </div>
  );
}
