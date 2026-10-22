import { useCallback, useEffect, useRef, useState } from 'react';

import {
  clearSessions,
  deleteSession,
  isTauri,
  listSessions,
} from '../api/ipc';
import { ipcErrorMessage } from '../lib/ipcError';
import type { SessionSummary } from '../types';

export function useSessionHistory() {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }

    setLoading(true);
    try {
      const list = await listSessions();
      if (mounted.current) {
        setSessions(list);
        setError(null);
      }
    } catch (loadError) {
      if (mounted.current) {
        setError(ipcErrorMessage(loadError));
      }
    } finally {
      if (mounted.current) {
        setLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    mounted.current = true;
    void refresh();
    return () => {
      mounted.current = false;
    };
  }, [refresh]);

  const remove = useCallback(
    async (id: string) => {
      try {
        await deleteSession(id);
        await refresh();
      } catch (deleteError) {
        setError(ipcErrorMessage(deleteError));
      }
    },
    [refresh],
  );

  const clear = useCallback(async () => {
    try {
      await clearSessions();
      await refresh();
    } catch (clearError) {
      setError(ipcErrorMessage(clearError));
    }
  }, [refresh]);

  return { sessions, loading, error, refresh, remove, clear };
}
