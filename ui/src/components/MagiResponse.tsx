import type { MagiState, Phase } from '../hooks/useMagiSession';
import type { DecisionStatus, MagiVerdict } from '../types';

interface ResponseMeta {
  text: string;
  color: string;
  sub: string;
  flicker: boolean;
}

const GREEN = 'var(--color-term-green)';
const RED = 'var(--color-term-red)';
const AMBER = 'var(--color-term-amber)';
const BLUE = 'var(--color-term-blue)';
const GRAY = 'var(--color-magi-gray)';

function decisionMeta(status: DecisionStatus): ResponseMeta {
  switch (status) {
    case 'yes':
      return { text: '合 意', color: GREEN, sub: 'YES', flicker: false };
    case 'no':
      return { text: '拒 絶', color: RED, sub: 'NO', flicker: false };
    case 'conditional':
      return { text: '状 態', color: AMBER, sub: 'CONDITIONAL', flicker: false };
    case 'error':
      return { text: '誤 差', color: GRAY, sub: 'ERROR', flicker: false };
    case 'info':
      return { text: '情 報', color: BLUE, sub: 'INFO', flicker: false };
  }
}

function finalMeta(verdict: MagiVerdict): ResponseMeta {
  switch (verdict.kind) {
    case 'unanimous':
      if (verdict.stance === 'approve') {
        return { text: '合 意', color: GREEN, sub: 'FINAL // 3:0', flicker: false };
      }
      if (verdict.stance === 'reject') {
        return { text: '拒 絶', color: RED, sub: 'FINAL // 3:0', flicker: false };
      }
      return {
        text: '情 報',
        color: BLUE,
        sub: 'FINAL // 3:0 ABSTAIN',
        flicker: false,
      };
    case 'majority':
      if (verdict.stance === 'approve') {
        return { text: '状 態', color: AMBER, sub: 'FINAL // 2:1', flicker: false };
      }
      return { text: '拒 絶', color: RED, sub: 'FINAL // 2:1', flicker: false };
    case 'split':
      return { text: '情 報', color: BLUE, sub: 'FINAL // SPLIT', flicker: false };
    case 'incomplete':
      return {
        text: '誤 差',
        color: GRAY,
        sub: 'FINAL // INCOMPLETE',
        flicker: false,
      };
  }
}

function phaseMeta(phase: Phase): ResponseMeta {
  switch (phase) {
    case 'opening':
      return { text: '情 報', color: BLUE, sub: 'ANALYZING', flicker: true };
    case 'debating':
      return { text: '情 報', color: BLUE, sub: 'DEBATE', flicker: true };
    case 'voting':
      return { text: '情 報', color: BLUE, sub: 'SEALING BALLOTS', flicker: true };
    case 'opening_ready':
      return { text: '情 報', color: BLUE, sub: 'OPENING READY', flicker: false };
    case 'debate_ready':
      return { text: '情 報', color: BLUE, sub: 'READY TO VOTE', flicker: false };
    case 'verdict':
      return { text: '情 報', color: BLUE, sub: 'VERDICT', flicker: false };
    default:
      return { text: '情 報', color: BLUE, sub: 'STANDBY', flicker: false };
  }
}

function responseMeta(state: MagiState): ResponseMeta {
  if (state.stageError && !state.votes) {
    return { text: '誤 差', color: GRAY, sub: 'ERROR', flicker: false };
  }
  if (state.votes) {
    return finalMeta(state.votes.verdict);
  }
  if (state.openings) {
    const provisional = decisionMeta(state.openings.decision);
    return { ...provisional, sub: `PROVISIONAL // ${provisional.sub}` };
  }
  return phaseMeta(state.phase);
}

export default function MagiResponse({ state }: { state: MagiState }) {
  const meta = responseMeta(state);

  return (
    <div
      className={`magi-response ${meta.flicker ? 'magi-flicker' : ''}`}
      style={{ color: meta.color }}
    >
      <div className="magi-response-inner">{meta.text}</div>
      <div className="magi-response-sub">{meta.sub}</div>
    </div>
  );
}
