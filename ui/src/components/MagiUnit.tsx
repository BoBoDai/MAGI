import type { NodeStatus } from '../hooks/useMagiSession';
import { PERSONA_META, unitName } from '../lib/labels';
import type { FinalVote, Opening, PersonaId, Stance } from '../types';

interface MagiUnitProps {
  persona: PersonaId;
  status: NodeStatus;
  opening: Opening | null;
  vote: FinalVote | null;
  onClick: () => void;
}

interface UnitVisual {
  fill: string;
  text: string;
  label: string;
  flicker: boolean;
}

function visual(status: NodeStatus, stance: Stance | null): UnitVisual {
  if (status === 'failed') {
    return { fill: '#000000', text: '#ff4d4d', label: 'ERROR', flicker: false };
  }
  if (status === 'thinking') {
    return {
      fill: 'var(--color-term-amber)',
      text: '#000000',
      label: 'THINKING',
      flicker: true,
    };
  }

  switch (stance) {
    case 'approve':
      return {
        fill: 'var(--color-term-green)',
        text: '#000000',
        label: 'APPROVE',
        flicker: false,
      };
    case 'reject':
      return {
        fill: 'var(--color-magi-red)',
        text: '#f0e6d2',
        label: 'REJECT',
        flicker: false,
      };
    case 'abstain':
      return {
        fill: 'var(--color-term-blue)',
        text: '#000000',
        label: 'ABSTAIN',
        flicker: false,
      };
    default:
      return {
        fill: '#120c04',
        text: 'var(--color-term-amber)',
        label: 'STANDBY',
        flicker: false,
      };
  }
}

export default function MagiUnit({
  persona,
  status,
  opening,
  vote,
  onClick,
}: MagiUnitProps) {
  const meta = PERSONA_META[persona];
  const stance = vote?.stance ?? opening?.stance ?? null;
  const confidence = vote?.confidence ?? opening?.confidence ?? null;
  const style = visual(status, stance);
  const statusLine =
    confidence !== null && (stance === 'approve' || stance === 'reject' || stance === 'abstain')
      ? `${style.label} ${confidence}`
      : style.label;

  return (
    <button
      type="button"
      className={`magi-unit ${persona}`}
      style={{
        background:
          status === 'failed'
            ? 'var(--color-magi-red)'
            : 'var(--color-term-amber)',
      }}
      onClick={onClick}
      aria-label={`${meta.unit} ${meta.label} ${style.label}`}
      title={`${meta.unit} // ${meta.label}`}
    >
      <div
        className={`magi-unit-inner ${style.flicker ? 'magi-flicker' : ''}`}
        style={{ background: style.fill, color: style.text }}
      >
        <span>{unitName(persona)}</span>
        <span className="magi-unit-status">{statusLine}</span>
      </div>
    </button>
  );
}
