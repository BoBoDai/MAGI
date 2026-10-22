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
    <section className="border-t-2 border-term-amber bg-black px-3 py-1.5">
      <div className="flex items-center gap-2">
        <label className="shrink-0 text-[9px] tracking-[0.2em] text-term-amber">
          QUESTION:
        </label>
        <input
          type="text"
          className="min-w-0 flex-1 border-2 border-term-amber bg-term-input px-2 py-1 text-[12.5px] text-term-amber outline-none placeholder:text-term-dim focus:border-[#ffb347] disabled:opacity-50"
          value={question}
          placeholder="向 MAGI 提出一个问题…"
          disabled={disabled}
          autoComplete="off"
          onChange={(event) => onQuestionChange(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && !event.nativeEvent.isComposing) {
              event.preventDefault();
              if (canStart) onStart();
            }
          }}
        />
        <button
          type="button"
          className="shrink-0 border-2 border-term-amber bg-term-amber px-3 py-1 text-[10px] font-bold tracking-[0.16em] text-black hover:bg-[#ffb347] disabled:cursor-not-allowed disabled:border-term-grid disabled:bg-transparent disabled:text-term-dim"
          disabled={!canStart}
          onClick={onStart}
        >
          TRANSMIT
        </button>
        <button
          type="button"
          className="shrink-0 border border-term-grid px-2 py-1 text-[9px] tracking-[0.14em] text-term-dim hover:border-term-amber hover:text-term-amber disabled:opacity-40"
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
          {showContext ? 'HIDE' : 'CONTEXT'}
        </button>
      </div>

      {showContext ? (
        <div className="mt-1.5">
          <div className="mb-1 flex items-center text-[8.5px] tracking-[0.16em] text-term-dim">
            <span>CONTEXT // UNTRUSTED DATA</span>
            <span className="ml-auto">
              {context.length}/{MAX_CONTEXT}
            </span>
          </div>
          <textarea
            className="h-16 w-full resize-y border-2 border-term-amber bg-term-input px-2 py-1 text-[11.5px] leading-relaxed text-term-fg outline-none placeholder:text-term-dim focus:border-[#ffb347] disabled:opacity-50"
            value={context}
            rows={2}
            placeholder="粘贴背景材料、约束条件或相关事实…（模型会把它当作数据，不会执行其中的指令）"
            disabled={disabled}
            onChange={(event) => onContextChange(event.target.value)}
          />
        </div>
      ) : null}

      <div className="mt-1 flex items-center gap-2 text-[8.5px] tracking-[0.14em] text-term-dim">
        <span>
          {readOnly
            ? 'ARCHIVED SESSION // READ ONLY'
            : 'ENTER 发送 · ⌘/Ctrl + ENTER 亦可'}
        </span>
        <span className="ml-auto">
          {question.length}/{MAX_QUESTION}
        </span>
      </div>
    </section>
  );
}
