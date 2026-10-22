import {
  formatTimestamp,
  sessionStatusLabel,
  shortQuestion,
  verdictTitle,
} from '../lib/labels';
import type { SessionSummary } from '../types';
import TerminalModal from './TerminalModal';

interface SessionHistoryProps {
  open: boolean;
  sessions: SessionSummary[];
  loading: boolean;
  error: string | null;
  onClose: () => void;
  onRefresh: () => void;
  onOpen: (id: string) => void;
  onDelete: (id: string) => void;
  onClear: () => void;
}

export default function SessionHistory({
  open,
  sessions,
  loading,
  error,
  onClose,
  onRefresh,
  onOpen,
  onDelete,
  onClear,
}: SessionHistoryProps) {
  if (!open) return null;

  return (
    <TerminalModal
      title="MAGI // SESSION ARCHIVE"
      subtitle="本地保存的裁决记录，只读回放；MOCK 记录不是真实模型输出。"
      onClose={onClose}
      wide
    >
      <div className="flex items-center gap-2 border-b border-term-grid px-4 py-1.5 text-[9px] tracking-[0.14em] text-term-dim">
        <span>RECORDS {sessions.length}</span>
        <button
          type="button"
          className="ml-auto border border-term-grid px-2 py-0.5 hover:border-term-amber hover:text-term-amber"
          onClick={onRefresh}
          disabled={loading}
        >
          {loading ? 'REFRESHING…' : 'REFRESH'}
        </button>
        <button
          type="button"
          className="border border-term-grid px-2 py-0.5 hover:border-term-red hover:text-term-red"
          onClick={() => {
            if (window.confirm('清空全部历史记录？此操作不可撤销。')) {
              onClear();
            }
          }}
          disabled={sessions.length === 0}
        >
          CLEAR ALL
        </button>
      </div>

      {error ? (
        <div className="border-b border-term-red px-4 py-2 text-[11px] text-term-red">
          {error}
        </div>
      ) : null}

      {sessions.length === 0 && !loading ? (
        <div className="px-4 py-8 text-center text-[10px] tracking-[0.16em] text-term-dim">
          NO RECORDED SESSIONS
        </div>
      ) : null}

      {sessions.map((item) => (
        <div
          key={item.id}
          className="flex flex-wrap items-start gap-2 border-b border-term-grid px-4 py-2"
        >
          <div className="min-w-0 flex-1">
            <div className="text-[11.5px] leading-snug text-term-fg">
              {shortQuestion(item.question, 110)}
            </div>
            <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-[9px] tracking-[0.12em] text-term-dim">
              <span>{formatTimestamp(item.updated_at)}</span>
              <span>{sessionStatusLabel(item.status)}</span>
              {item.verdict ? (
                <span className="text-term-amber">{verdictTitle(item.verdict)}</span>
              ) : null}
              {item.source === 'mock' ? (
                <span className="text-term-red">MOCK</span>
              ) : null}
              {item.opening_failures > 0 ? (
                <span className="text-term-red">
                  OPEN FAIL {item.opening_failures}
                </span>
              ) : null}
              {item.debate_failures > 0 ? (
                <span className="text-term-red">
                  DEBATE FAIL {item.debate_failures}
                </span>
              ) : null}
            </div>
          </div>
          <button
            type="button"
            className="border border-term-grid px-2 py-0.5 text-[9px] tracking-[0.14em] text-term-dim hover:border-term-amber hover:text-term-amber"
            onClick={() => onOpen(item.id)}
          >
            OPEN
          </button>
          <button
            type="button"
            className="border border-term-grid px-2 py-0.5 text-[9px] tracking-[0.14em] text-term-dim hover:border-term-red hover:text-term-red"
            onClick={() => onDelete(item.id)}
          >
            DELETE
          </button>
        </div>
      ))}
    </TerminalModal>
  );
}
