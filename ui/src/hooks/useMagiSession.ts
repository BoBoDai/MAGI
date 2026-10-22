import { useCallback, useReducer, useRef } from 'react';

import { castVotes, runDebate, runOpening, saveSession } from '../api/ipc';
import { ipcErrorMessage } from '../lib/ipcError';
import type {
  DebateBatch,
  MagiStreamEvent,
  OpeningBatch,
  PersonaId,
  SessionDraft,
  SessionRecord,
  Stage,
  VoteBatch,
} from '../types';

export type Phase =
  | 'idle'
  | 'opening'
  | 'opening_ready'
  | 'debating'
  | 'debate_ready'
  | 'voting'
  | 'verdict';

export type NodeStatus = 'idle' | 'thinking' | 'done' | 'failed';

export interface MagiState {
  phase: Phase;
  readOnly: boolean;
  sessionId: string;
  runId: string;
  question: string;
  context: string;
  openings: OpeningBatch | null;
  debate: DebateBatch | null;
  votes: VoteBatch | null;
  streaming: Record<PersonaId, string>;
  nodes: Record<PersonaId, NodeStatus>;
  nodeErrors: Record<PersonaId, string | null>;
  stageError: string | null;
  failedStage: Stage | null;
  saveWarning: string | null;
}

type Action =
  | {
      type: 'START';
      sessionId: string;
      runId: string;
      question: string;
      context: string;
    }
  | { type: 'STREAM'; event: MagiStreamEvent }
  | { type: 'OPENING_DONE'; batch: OpeningBatch }
  | { type: 'DEBATE_START'; runId: string }
  | { type: 'DEBATE_DONE'; batch: DebateBatch }
  | { type: 'VOTE_START' }
  | { type: 'VOTE_DONE'; batch: VoteBatch }
  | { type: 'STAGE_ERROR'; stage: Stage; message: string }
  | { type: 'SAVE_WARNING'; message: string | null }
  | { type: 'LOAD'; record: SessionRecord }
  | { type: 'RESET' };

function emptyStrings(): Record<PersonaId, string> {
  return { melchior: '', balthasar: '', casper: '' };
}

function emptyNodes(status: NodeStatus = 'idle'): Record<PersonaId, NodeStatus> {
  return { melchior: status, balthasar: status, casper: status };
}

function emptyErrors(): Record<PersonaId, string | null> {
  return { melchior: null, balthasar: null, casper: null };
}

const initialState: MagiState = {
  phase: 'idle',
  readOnly: false,
  sessionId: '',
  runId: '',
  question: '',
  context: '',
  openings: null,
  debate: null,
  votes: null,
  streaming: emptyStrings(),
  nodes: emptyNodes(),
  nodeErrors: emptyErrors(),
  stageError: null,
  failedStage: null,
  saveWarning: null,
};

function nodesFromOpenings(batch: OpeningBatch): {
  nodes: Record<PersonaId, NodeStatus>;
  errors: Record<PersonaId, string | null>;
} {
  const nodes = emptyNodes();
  const errors = emptyErrors();

  for (const outcome of batch.outcomes) {
    if (outcome.status === 'ok') {
      nodes[outcome.opening.persona] = 'done';
    } else {
      nodes[outcome.persona] = 'failed';
      errors[outcome.persona] = outcome.message;
    }
  }

  return { nodes, errors };
}

function nodesFromVotes(batch: VoteBatch): {
  nodes: Record<PersonaId, NodeStatus>;
  errors: Record<PersonaId, string | null>;
} {
  const nodes = emptyNodes();
  const errors = emptyErrors();

  for (const outcome of batch.outcomes) {
    if (outcome.status === 'ok') {
      nodes[outcome.vote.persona] = 'done';
    } else {
      nodes[outcome.persona] = 'failed';
      errors[outcome.persona] = outcome.message;
    }
  }

  return { nodes, errors };
}

function reducer(state: MagiState, action: Action): MagiState {
  switch (action.type) {
    case 'START':
      return {
        ...initialState,
        phase: 'opening',
        sessionId: action.sessionId,
        runId: action.runId,
        question: action.question,
        context: action.context,
      };

    case 'STREAM': {
      const event = action.event;
      if (event.run_id !== state.runId) return state;

      switch (event.kind) {
        case 'persona_started':
          return {
            ...state,
            nodes: { ...state.nodes, [event.persona]: 'thinking' },
          };
        case 'delta':
          return {
            ...state,
            streaming: {
              ...state.streaming,
              [event.persona]: state.streaming[event.persona] + event.text,
            },
          };
        case 'persona_completed':
          return {
            ...state,
            nodes: { ...state.nodes, [event.persona]: 'done' },
          };
        case 'persona_failed':
          return {
            ...state,
            nodes: { ...state.nodes, [event.persona]: 'failed' },
            nodeErrors: {
              ...state.nodeErrors,
              [event.persona]: event.message,
            },
          };
        default:
          return state;
      }
    }

    case 'OPENING_DONE': {
      const { nodes, errors } = nodesFromOpenings(action.batch);
      return {
        ...state,
        phase: 'opening_ready',
        openings: action.batch,
        streaming: emptyStrings(),
        nodes,
        nodeErrors: errors,
        stageError: null,
        failedStage: null,
      };
    }

    case 'DEBATE_START':
      return {
        ...state,
        phase: 'debating',
        runId: action.runId,
        debate: null,
        streaming: emptyStrings(),
        nodes: emptyNodes(),
        nodeErrors: emptyErrors(),
        stageError: null,
        failedStage: null,
      };

    case 'DEBATE_DONE':
      return {
        ...state,
        phase: 'debate_ready',
        debate: action.batch,
        streaming: emptyStrings(),
        stageError: null,
        failedStage: null,
      };

    case 'VOTE_START':
      return {
        ...state,
        phase: 'voting',
        votes: null,
        streaming: emptyStrings(),
        nodes: emptyNodes('thinking'),
        nodeErrors: emptyErrors(),
        stageError: null,
        failedStage: null,
      };

    case 'VOTE_DONE': {
      const { nodes, errors } = nodesFromVotes(action.batch);
      return {
        ...state,
        phase: 'verdict',
        votes: action.batch,
        nodes,
        nodeErrors: errors,
        stageError: null,
        failedStage: null,
      };
    }

    case 'STAGE_ERROR':
      return {
        ...state,
        phase:
          action.stage === 'opening'
            ? 'idle'
            : action.stage === 'debate'
              ? 'opening_ready'
              : state.debate
                ? 'debate_ready'
                : 'opening_ready',
        nodes: emptyNodes(),
        streaming: emptyStrings(),
        stageError: action.message,
        failedStage: action.stage,
      };

    case 'SAVE_WARNING':
      return { ...state, saveWarning: action.message };

    case 'LOAD': {
      const record = action.record;
      const phase: Phase = record.vote
        ? 'verdict'
        : record.debate
          ? 'debate_ready'
          : 'opening_ready';
      const { nodes, errors } = record.vote
        ? nodesFromVotes(record.vote)
        : record.openings
          ? nodesFromOpenings(record.openings)
          : { nodes: emptyNodes(), errors: emptyErrors() };

      return {
        ...initialState,
        phase,
        readOnly: true,
        sessionId: record.id,
        question: record.question,
        context: record.context,
        openings: record.openings,
        debate: record.debate,
        votes: record.vote,
        nodes,
        nodeErrors: errors,
      };
    }

    case 'RESET':
      return initialState;
  }
}

function newId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export interface MagiActions {
  start: (question: string, context: string) => Promise<void>;
  debate: () => Promise<void>;
  vote: () => Promise<void>;
  reset: () => void;
  load: (record: SessionRecord) => void;
}

export function useMagiSession(onSaved?: () => void): {
  state: MagiState;
  actions: MagiActions;
} {
  const [state, dispatch] = useReducer(reducer, initialState);
  const runIdRef = useRef('');

  const persist = useCallback(
    async (draft: SessionDraft, runId: string) => {
      try {
        await saveSession(draft);
        if (runIdRef.current === runId) {
          dispatch({ type: 'SAVE_WARNING', message: null });
        }
        onSaved?.();
      } catch (error) {
        if (runIdRef.current === runId) {
          dispatch({
            type: 'SAVE_WARNING',
            message: `历史保存失败：${ipcErrorMessage(error)}`,
          });
        }
      }
    },
    [onSaved],
  );

  const start = useCallback(
    async (question: string, context: string) => {
      const trimmed = question.trim();
      if (!trimmed) return;

      const runId = newId();
      const sessionId = newId();
      runIdRef.current = runId;
      dispatch({
        type: 'START',
        sessionId,
        runId,
        question: trimmed,
        context: context.trim(),
      });

      try {
        const batch = await runOpening(
          { run_id: runId, question: trimmed, context: context.trim() },
          (event) => {
            if (event.run_id === runIdRef.current) {
              dispatch({ type: 'STREAM', event });
            }
          },
        );

        if (runIdRef.current !== runId) return;
        dispatch({ type: 'OPENING_DONE', batch });
        void persist(
          {
            id: sessionId,
            question: trimmed,
            context: context.trim(),
            openings: batch,
            debate: null,
            vote: null,
          },
          runId,
        );
      } catch (error) {
        if (runIdRef.current !== runId) return;
        dispatch({
          type: 'STAGE_ERROR',
          stage: 'opening',
          message: ipcErrorMessage(error),
        });
      }
    },
    [persist],
  );

  const debate = useCallback(async () => {
    if (!state.openings || state.phase !== 'opening_ready') return;

    const runId = newId();
    runIdRef.current = runId;
    dispatch({ type: 'DEBATE_START', runId });

    try {
      const batch = await runDebate(
        {
          run_id: runId,
          question: state.question,
          context: state.context,
          openings: state.openings,
        },
        (event) => {
          if (event.run_id === runIdRef.current) {
            dispatch({ type: 'STREAM', event });
          }
        },
      );

      if (runIdRef.current !== runId) return;
      dispatch({ type: 'DEBATE_DONE', batch });
      void persist(
        {
          id: state.sessionId,
          question: state.question,
          context: state.context,
          openings: state.openings,
          debate: batch,
          vote: null,
        },
        runId,
      );
    } catch (error) {
      if (runIdRef.current !== runId) return;
      dispatch({
        type: 'STAGE_ERROR',
        stage: 'debate',
        message: ipcErrorMessage(error),
      });
    }
  }, [persist, state]);

  const vote = useCallback(async () => {
    if (!state.openings) return;
    if (state.phase !== 'opening_ready' && state.phase !== 'debate_ready') {
      return;
    }

    const runId = runIdRef.current;
    dispatch({ type: 'VOTE_START' });

    try {
      const batch = await castVotes({
        question: state.question,
        context: state.context,
        openings: state.openings,
        debate: state.debate,
      });

      if (runIdRef.current !== runId) return;
      dispatch({ type: 'VOTE_DONE', batch });
      void persist(
        {
          id: state.sessionId,
          question: state.question,
          context: state.context,
          openings: state.openings,
          debate: state.debate,
          vote: batch,
        },
        runId,
      );
    } catch (error) {
      if (runIdRef.current !== runId) return;
      dispatch({
        type: 'STAGE_ERROR',
        stage: 'vote',
        message: ipcErrorMessage(error),
      });
    }
  }, [persist, state]);

  const reset = useCallback(() => {
    runIdRef.current = '';
    dispatch({ type: 'RESET' });
  }, []);

  const load = useCallback((record: SessionRecord) => {
    runIdRef.current = '';
    dispatch({ type: 'LOAD', record });
  }, []);

  return {
    state,
    actions: { start, debate, vote, reset, load },
  };
}
