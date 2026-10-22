import { useCallback, useEffect, useState } from 'react';

import {
  getLlmSettings,
  getRuntimeFlags,
  getSession,
  isTauri,
  setDemoMode,
} from './api/ipc';
import AppHeader from './components/AppHeader';
import MagiConsole from './components/MagiConsole';
import QuestionComposer from './components/QuestionComposer';
import SessionHistory from './components/SessionHistory';
import SettingsPanel from './components/SettingsPanel';
import { useMagiSession } from './hooks/useMagiSession';
import { useSessionHistory } from './hooks/useSessionHistory';
import { ipcErrorMessage } from './lib/ipcError';
import type { LlmSettingsView, RuntimeFlags } from './types';

export default function App() {
  const inTauri = isTauri();
  const [settings, setSettings] = useState<LlmSettingsView | null>(null);
  const [flags, setFlags] = useState<RuntimeFlags | null>(null);
  const [demoEnabled, setDemoEnabled] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [question, setQuestion] = useState('');
  const [context, setContext] = useState('');
  const [loadError, setLoadError] = useState<string | null>(null);
  const [demoError, setDemoError] = useState<string | null>(null);

  const history = useSessionHistory();
  const { state, actions } = useMagiSession(history.refresh);

  useEffect(() => {
    if (!inTauri) return;

    let cancelled = false;
    Promise.all([getLlmSettings(), getRuntimeFlags()])
      .then(([loadedSettings, loadedFlags]) => {
        if (cancelled) return;
        setSettings(loadedSettings);
        setFlags(loadedFlags);
        setDemoEnabled(loadedFlags.demo_enabled);
      })
      .catch((error: unknown) => {
        if (!cancelled) setLoadError(ipcErrorMessage(error));
      });

    return () => {
      cancelled = true;
    };
  }, [inTauri]);

  const busy =
    state.phase === 'opening' ||
    state.phase === 'debating' ||
    state.phase === 'voting';

  const handleStart = useCallback(() => {
    void actions.start(question, context);
  }, [actions, question, context]);

  const handleNewSession = useCallback(() => {
    actions.reset();
    setQuestion('');
    setContext('');
  }, [actions]);

  const handleOpenSession = useCallback(
    async (id: string) => {
      try {
        const record = await getSession(id);
        actions.load(record);
        setQuestion(record.question);
        setContext(record.context);
        setHistoryOpen(false);
      } catch (error) {
        setLoadError(ipcErrorMessage(error));
      }
    },
    [actions],
  );

  const handleToggleDemo = useCallback(async () => {
    const next = !demoEnabled;
    setDemoError(null);

    try {
      await setDemoMode(next);
      setDemoEnabled(next);
      setFlags((current) =>
        current ? { ...current, demo_enabled: next } : current,
      );
    } catch (error) {
      setDemoError(ipcErrorMessage(error));
    }
  }, [demoEnabled]);

  const handleSettingsSaved = useCallback((saved: LlmSettingsView) => {
    setSettings(saved);
    setSettingsOpen(false);
  }, []);

  if (!inTauri) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 bg-term-bg p-6 font-mono text-term-fg">
        <div className="text-[11px] tracking-[0.32em] text-term-amber">
          MAGI SYSTEM
        </div>
        <div className="max-w-md text-center text-[12px] leading-relaxed text-term-dim">
          未在 Tauri 中运行。纯浏览器模式没有 IPC，无法调用三脑协议。
          请使用 <code className="text-term-fg">pnpm app:dev</code> 启动桌面窗口。
        </div>
      </div>
    );
  }

  return (
    <div className="relative flex h-full flex-col overflow-hidden bg-term-bg font-mono text-term-fg">
      <div className="magi-scanlines pointer-events-none fixed inset-0 z-30" />

      <div className="relative z-10 flex h-full min-h-0 flex-col">
        <AppHeader
          settings={settings}
          flags={flags}
          demoEnabled={demoEnabled}
          busy={busy}
          onToggleDemo={() => void handleToggleDemo()}
          onOpenSettings={() => setSettingsOpen(true)}
          onOpenHistory={() => setHistoryOpen(true)}
        />

        {demoEnabled ? (
          <div className="border-b border-term-amber bg-term-amber/15 px-4 py-1 text-center text-[9.5px] tracking-[0.24em] text-term-amber">
            MOCK MODE // NOT REAL MODEL OUTPUT
          </div>
        ) : null}

        {loadError || demoError ? (
          <div className="border-b border-term-red px-4 py-1 text-[10px] text-term-red">
            {loadError ?? demoError}
          </div>
        ) : null}

        <QuestionComposer
          question={question}
          context={context}
          busy={busy}
          readOnly={state.readOnly}
          onQuestionChange={setQuestion}
          onContextChange={setContext}
          onStart={handleStart}
        />

        <MagiConsole
          state={state}
          actions={actions}
          onNewSession={handleNewSession}
        />
      </div>

      <SettingsPanel
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        onSaved={handleSettingsSaved}
      />

      <SessionHistory
        open={historyOpen}
        sessions={history.sessions}
        loading={history.loading}
        error={history.error}
        onClose={() => setHistoryOpen(false)}
        onRefresh={() => void history.refresh()}
        onOpen={(id) => void handleOpenSession(id)}
        onDelete={(id) => void history.remove(id)}
        onClear={() => void history.clear()}
      />
    </div>
  );
}
