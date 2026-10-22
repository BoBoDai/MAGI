import type { MagiVerdict, PersonaId, SessionStatus, Stance } from '../types';

export const PERSONAS: PersonaId[] = ['melchior', 'balthasar', 'casper'];

export const PERSONA_META: Record<
  PersonaId,
  { unit: string; short: string; number: number; label: string; lens: string }
> = {
  melchior: {
    unit: 'MELCHIOR-1',
    short: 'MELCHIOR',
    number: 1,
    label: '科学家',
    lens: '证据 · 方法 · 风险',
  },
  balthasar: {
    unit: 'BALTHASAR-2',
    short: 'BALTHASAR',
    number: 2,
    label: '母亲',
    lens: '保护 · 长期 · 可逆',
  },
  casper: {
    unit: 'CASPER-3',
    short: 'CASPER',
    number: 3,
    label: '女性',
    lens: '人 · 关系 · 直觉',
  },
};

export function unitName(persona: PersonaId): string {
  const meta = PERSONA_META[persona];
  return `${meta.short} • ${meta.number}`;
}

export const STANCE_LABEL: Record<Stance, string> = {
  approve: 'APPROVE',
  reject: 'REJECT',
  abstain: 'ABSTAIN',
};

export const STANCE_TEXT: Record<Stance, string> = {
  approve: 'text-term-green',
  reject: 'text-term-red',
  abstain: 'text-term-blue',
};

export const STANCE_BORDER: Record<Stance, string> = {
  approve: 'border-term-green',
  reject: 'border-term-red',
  abstain: 'border-term-blue',
};

export const STANCE_LED: Record<Stance, string> = {
  approve: 'bg-term-green',
  reject: 'bg-term-red',
  abstain: 'bg-term-blue',
};

export function verdictTitle(verdict: MagiVerdict): string {
  switch (verdict.kind) {
    case 'unanimous':
      return `UNANIMOUS 3:0 · ${STANCE_LABEL[verdict.stance]}`;
    case 'majority':
      return `MAJORITY 2:1 · ${STANCE_LABEL[verdict.stance]}`;
    case 'split':
      return 'SPLIT · NO MAJORITY';
    case 'incomplete':
      return 'INCOMPLETE · NODE FAILURE';
  }
}

export function verdictExplanation(verdict: MagiVerdict): string {
  switch (verdict.kind) {
    case 'unanimous':
      return verdict.stance === 'abstain'
        ? '三脑一致保留：现有信息不足以给出建议。'
        : '三脑一致，裁决已锁定。';
    case 'majority':
      return '两票形成多数，少数票的意见列在下方。';
    case 'split':
      return '没有形成多数，MAGI 无法给出单一建议。';
    case 'incomplete':
      return '至少一个脑未能完成终审，裁决不成立。';
  }
}

export function sessionStatusLabel(status: SessionStatus): string {
  switch (status) {
    case 'opening':
      return 'OPENING';
    case 'debate':
      return 'DEBATE';
    case 'vote':
      return 'VERDICT';
  }
}

export function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;

  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function shortQuestion(question: string, max = 72): string {
  const trimmed = question.trim().replace(/\s+/g, ' ');
  return trimmed.length > max ? `${trimmed.slice(0, max)}…` : trimmed;
}
