import type { LlmSettingsView, RuntimeFlags } from '../types';

interface AppHeaderProps {
  settings: LlmSettingsView | null;
  flags: RuntimeFlags | null;
  demoEnabled: boolean;
  busy: boolean;
  onToggleDemo: () => void;
  onOpenSettings: () => void;
  onOpenHistory: () => void;
}

const buttonClass =
  'border border-term-grid px-2 py-1 text-[9.5px] tracking-[0.14em] text-term-dim hover:border-term-amber hover:text-term-amber disabled:opacity-40';

export default function AppHeader({
  settings,
  flags,
  demoEnabled,
  busy,
  onToggleDemo,
  onOpenSettings,
  onOpenHistory,
}: AppHeaderProps) {
  const active =
    settings?.providers.find((provider) => provider.id === settings.active_id) ??
    settings?.providers[0] ??
    null;
  const online = demoEnabled || Boolean(active?.has_api_key);

  return (
    <header className="relative z-10 flex flex-wrap items-center gap-3 border-b border-term-grid bg-term-panel px-4 py-2">
      <div className="flex items-center gap-2">
        <span
          className={`h-2 w-2 ${
            online ? 'bg-term-green' : 'bg-term-red animate-pulse'
          }`}
        />
        <div>
          <div className="text-[10px] tracking-[0.3em] text-term-amber">
            MAGI SYSTEM
          </div>
          <div className="text-[9px] tracking-[0.18em] text-term-dim">
            DECISION PROTOCOL // THREE-BRAIN
          </div>
        </div>
      </div>

      <div className="ml-auto flex flex-wrap items-center gap-2 text-[9.5px] tracking-[0.14em] text-term-dim">
        <span className="border border-term-grid px-2 py-1">
          {active
            ? `${active.provider.toUpperCase()} // ${active.model}`
            : 'NO PROVIDER'}
        </span>
        <span className={online ? 'text-term-green' : 'text-term-red'}>
          {demoEnabled ? 'MOCK' : online ? 'LINKED' : 'NO KEY'}
        </span>
        {flags?.demo_available ? (
          <button
            type="button"
            className={buttonClass}
            onClick={onToggleDemo}
            disabled={busy}
          >
            {demoEnabled ? 'MOCK ON' : 'MOCK OFF'}
          </button>
        ) : null}
        <button
          type="button"
          className={buttonClass}
          onClick={onOpenHistory}
          disabled={busy}
        >
          HISTORY
        </button>
        <button
          type="button"
          className={buttonClass}
          onClick={onOpenSettings}
          disabled={busy}
        >
          SETTINGS
        </button>
        <span className="text-term-dim">v{flags?.version ?? '0.1.0'}</span>
      </div>
    </header>
  );
}
