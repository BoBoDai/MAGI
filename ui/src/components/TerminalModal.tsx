import { useEffect, type ReactNode } from 'react';

interface TerminalModalProps {
  title: string;
  subtitle?: string;
  onClose: () => void;
  children: ReactNode;
  wide?: boolean;
}

export default function TerminalModal({
  title,
  subtitle,
  onClose,
  children,
  wide = false,
}: TerminalModalProps) {
  useEffect(() => {
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };

    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/80 p-4"
      onMouseDown={onClose}
      role="presentation"
    >
      <div
        className={`flex max-h-[86vh] w-full flex-col border-2 border-term-amber bg-black font-mono text-term-fg ${
          wide ? 'max-w-5xl' : 'max-w-2xl'
        }`}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="flex items-start gap-3 border-b-2 border-term-amber px-4 py-2.5">
          <div className="min-w-0 flex-1">
            <h2 className="m-0 text-[10px] uppercase tracking-[0.26em] text-term-amber">
              {title}
            </h2>
            {subtitle ? (
              <p className="m-0 mt-1 text-[11px] text-term-dim">{subtitle}</p>
            ) : null}
          </div>
          <button
            type="button"
            className="px-2 py-0.5 text-base text-term-dim hover:text-term-fg"
            onClick={onClose}
            aria-label="关闭"
          >
            ✕
          </button>
        </header>
        <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
      </div>
    </div>
  );
}
