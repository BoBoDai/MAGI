import type { MagiState } from '../hooks/useMagiSession';
import { PERSONA_META, STANCE_LABEL } from '../lib/labels';
import { debateFor, openingFor, voteFor } from '../lib/sessionSelectors';
import type { PersonaId } from '../types';
import TerminalModal from './TerminalModal';
import UnitDetail from './UnitDetail';

interface MagiDetailModalProps {
  persona: PersonaId | null;
  state: MagiState;
  onClose: () => void;
}

export default function MagiDetailModal({
  persona,
  state,
  onClose,
}: MagiDetailModalProps) {
  if (!persona) return null;

  const meta = PERSONA_META[persona];
  const opening = openingFor(state.openings, persona);
  const debate = debateFor(state.debate, persona);
  const vote = voteFor(state.votes, persona);
  const error = state.nodeErrors[persona];
  const stance = vote?.stance ?? opening?.stance ?? null;
  const subtitle = `${stance ? `STANCE: ${STANCE_LABEL[stance]}` : 'STANDBY'} · ${meta.lens}`;

  return (
    <TerminalModal
      title={`${meta.unit} // ${meta.label}`}
      subtitle={subtitle}
      onClose={onClose}
      wide
    >
      <div className="p-4">
        <UnitDetail
          status={state.nodes[persona]}
          stream={state.streaming[persona]}
          opening={opening}
          debate={debate}
          vote={vote}
          error={error}
          phase={state.phase}
          readOnly={state.readOnly}
        />
      </div>
    </TerminalModal>
  );
}
