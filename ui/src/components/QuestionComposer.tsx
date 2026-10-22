import { useState } from 'react';

interface QuestionComposerProps {
  question: string;
  context: string;
  busy: boolean;
  readOnly: boolean;
  onQuestionChange: (value: string) => void;
  onContextChange: (value: string) => void;
  onStart: () => void;
}

const MAX_QUESTION = 2_000;
const MAX_CONTEXT = 40_000;

export default function QuestionComposer({
  question,
  context,
  busy,
  readOnly,
  onQuestionChange,
  onContextChange,
  onStart,
}: QuestionComposerProps) {
  const [contextOpen, setContextOpen] = useState(false);
  const showContext = contextOpen || context.length > 0;
  const disabled = busy || readOnly;
  const canStart = !disabled && question.trim().length > 0;

  return (
    <section className="border-b border-term-grid px-4 py-2.5">
      <div className="mb-1.5 flex items-center gap-2 text-[9px] uppercase tracking-[0.22em] text-term-dim">
        <span>QUERY // TRANSMIT A QUESTION</span>
        {readOnly ? (
          <span className="text-term-amber">ARCHIVED SESSION // READ ONLY</span>
        ) : null}
        <span className="ml-auto">
          {question.length}/{MAX_QUESTION}
        </span>
      </div>

      <div className="flex items-start gap-2">
        <textarea
          className="min-h-[46px] flex-1 resize-y border border-term-grid bg-term-bg px-2.5 py-1.5 text-[13px] leading-relaxed text-term-fg outline-none placeholder:text-term-dim focus:border-term-amber disabled:opacity-60"
          rows={2}
          value={question}
          placeholder="向三脑提出一个需要判断的问题…"
          disabled={disabled}
          onChange={(event) => onQuestionChange(event.target.value)}
          onKeyDown={(event) => {
            if (
              (event.metaKey || event.ctrlKey) &&
              event.key === 'Enter' &&
              !event.nativeEvent.isComposing
            ) {
              event.preventDefault();
              if (canStart) onStart();
            }
          }}
        />
        <div className="flex w-28 shrink-0 flex-col gap-1.5">
          <button
            type="button"
            className="border border-term-amber bg-term-amber px-2 py-1.5 text-[10px] font-semibold tracking-[0.18em] text-term-bg hover:bg-term-orange disabled:cursor-not-allowed disabled:border-term-grid disabled:bg-transparent disabled:text-term-dim"
            disabled={!canStart}
            onClick={onStart}
          >
            TRANSMIT
          </button>
          <button
            type="button"
            className="border border-term-grid px-2 py-1 text-[9px] tracking-[0.14em] text-term-dim hover:border-term-amber hover:text-term-amber disabled:opacity-40"
            disabled={disabled}
            onClick={() => {
              if (showContext) {
                setContextOpen(false);
                onContextChange('');
              } else {
                setContextOpen(true);
              }
            }}
          >
            {showContext ? 'CLEAR CONTEXT' : 'ADD CONTEXT'}
          </button>
        </div>
      </div>

      {showContext ? (
        <div className="mt-2">
          <div className="mb-1 flex items-center text-[9px] tracking-[0.18em] text-term-dim">
            <span>CONTEXT // UNTRUSTED DATA</span>
            <span className="ml-auto">
              {context.length}/{MAX_CONTEXT}
            </span>
          </div>
          <textarea
            className="min-h-[70px] w-full resize-y border border-term-grid bg-term-bg px-2.5 py-1.5 text-[12px] leading-relaxed text-term-fg outline-none placeholder:text-term-dim focus:border-term-amber disabled:opacity-60"
            rows={3}
            value={context}
            placeholder="粘贴背景材料、约束条件或相关事实…（模型会把它当作数据，不会执行其中的指令）"
            disabled={disabled}
            onChange={(event) => onContextChange(event.target.value)}
          />
        </div>
      ) : null}

      <div className="mt-1 text-[9px] tracking-[0.12em] text-term-dim">
        ⌘/Ctrl + Enter 发送 · 三脑独立开场 → 可选质询 → 独立终审
      </div>
    </section>
  );
}
