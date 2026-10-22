import type { NodeStatus, Phase } from '../hooks/useMagiSession';
import {
  PERSONA_META,
  STANCE_BORDER,
  STANCE_LABEL,
  STANCE_LED,
  STANCE_TEXT,
} from '../lib/labels';
import type { DebateReply, FinalVote, Opening, PersonaId, Stance } from '../types';

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

function ConfidenceBar({ value, stance }: { value: number; stance: Stance }) {
  const width = Math.min(100, Math.max(0, value));
  return (
    <div className="h-0.5 w-full bg-term-grid">
      <div
        className={`h-0.5 ${STANCE_LED[stance]}`}
        style={{ width: `${width}%` }}
      />
    </div>
  );
}

function Bullets({ title, items }: { title: string; items: string[] }) {
  return (
    <div>
      <div className="mb-0.5 text-[8.5px] tracking-[0.2em] text-term-dim">{title}</div>
      <ul className="m-0 list-none space-y-0.5 p-0 text-[11px] leading-relaxed">
        {items.map((item, index) => (
          <li key={`${index}-${item}`} className="flex gap-1.5">
            <span className="text-term-dim">›</span>
            <span>{item}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function ChangeBadge({ vote }: { vote: FinalVote }) {
  if (vote.changed_from_opening === null) {
    return (
      <span className="border border-term-grid px-1.5 py-0.5 text-[8.5px] tracking-[0.14em] text-term-dim">
        NO BASELINE
      </span>
    );
  }

  if (!vote.changed_from_opening) {
    return (
      <span className="border border-term-grid px-1.5 py-0.5 text-[8.5px] tracking-[0.14em] text-term-dim">
        STABLE
      </span>
    );
  }

  return (
    <span className="border border-term-amber px-1.5 py-0.5 text-[8.5px] tracking-[0.14em] text-term-amber">
      CHANGED
      {vote.opening_stance
        ? ` // ${STANCE_LABEL[vote.opening_stance]} → ${STANCE_LABEL[vote.stance]}`
        : ''}
    </span>
  );
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

  const showLive =
    stream.length > 0 &&
    ((phase === 'opening' && !opening) || (phase === 'debating' && !debate));

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
      </div>

      <div className="border-b border-term-grid px-3 py-1 text-[9.5px] tracking-[0.12em] text-term-dim">
        {meta.label} · {meta.lens}
      </div>

      <div className="min-h-0 flex-1 space-y-3 overflow-y-auto px-3 py-2">
        {error ? (
          <div className="whitespace-pre-wrap border border-term-red px-2 py-1.5 text-[11px] text-term-red">
            {error}
          </div>
        ) : null}

        {showLive ? (
          <div>
            <div className="mb-1 flex items-center gap-2 text-[8.5px] tracking-[0.2em] text-term-dim">
              <span>LIVE FEED</span>
              <span className="text-term-amber">
                {phase === 'opening' ? 'DECODING' : 'ARGUMENT'}
              </span>
            </div>
            <pre className="m-0 whitespace-pre-wrap break-words text-[10.5px] leading-relaxed text-term-dim">
              {stream}
              <span className="animate-pulse text-term-amber">▍</span>
            </pre>
          </div>
        ) : null}

        {status === 'thinking' && !showLive ? (
          <div className="text-[10px] tracking-[0.16em] text-term-amber">
            {phase === 'voting' ? 'BALLOT SEALED // AWAITING VERDICT' : 'PROCESSING…'}
          </div>
        ) : null}

        {opening ? (
          <section className="space-y-2">
            <div className="flex items-center gap-2">
              <span
                className={`border px-1.5 py-0.5 text-[9px] tracking-[0.14em] ${STANCE_BORDER[opening.stance]} ${STANCE_TEXT[opening.stance]}`}
              >
                {STANCE_LABEL[opening.stance]}
              </span>
              <span className="text-[9px] tracking-[0.12em] text-term-dim">
                CONF {opening.confidence}
              </span>
              <span className="ml-auto max-w-[45%] truncate text-[8.5px] text-term-dim">
                {opening.model}
              </span>
            </div>
            <ConfidenceBar value={opening.confidence} stance={opening.stance} />
            <p className="m-0 text-[12px] leading-relaxed text-term-fg">
              {opening.summary}
            </p>
            <Bullets title="KEY POINTS" items={opening.key_points} />
            <Bullets title="RISKS" items={opening.risks} />
          </section>
        ) : null}

        {debate ? (
          <section className="space-y-1 border-t border-term-grid pt-2">
            <div className="text-[8.5px] tracking-[0.2em] text-term-dim">
              CROSS-EXAMINATION
            </div>
            <p className="m-0 whitespace-pre-wrap text-[11.5px] leading-relaxed text-term-fg">
              {debate.text}
            </p>
            <div className="text-[8.5px] text-term-dim">{debate.model}</div>
          </section>
        ) : null}

        {vote ? (
          <section className="space-y-1.5 border-t border-term-grid pt-2">
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-[8.5px] tracking-[0.2em] text-term-dim">
                FINAL BALLOT
              </span>
              <span
                className={`border px-1.5 py-0.5 text-[9px] tracking-[0.14em] ${STANCE_BORDER[vote.stance]} ${STANCE_TEXT[vote.stance]}`}
              >
                {STANCE_LABEL[vote.stance]}
              </span>
              <span className="text-[9px] text-term-dim">CONF {vote.confidence}</span>
              <ChangeBadge vote={vote} />
            </div>
            <ConfidenceBar value={vote.confidence} stance={vote.stance} />
            <p className="m-0 text-[11.5px] leading-relaxed text-term-fg">
              {vote.reason}
            </p>
          </section>
        ) : null}

        {!error &&
        !showLive &&
        !opening &&
        !debate &&
        !vote &&
        status !== 'thinking' ? (
          <div className="text-[10px] tracking-[0.16em] text-term-dim">
            {readOnly ? 'NO DATA RECORDED' : 'STANDBY // AWAITING QUERY'}
          </div>
        ) : null}
      </div>
    </div>
  );
}
