/**
 * IPC wrapper. Components never call `invoke` directly.
 */

import { Channel, invoke } from '@tauri-apps/api/core';

import type {
  ConnectionTestResult,
  DebateBatch,
  DebateRequest,
  LlmModelList,
  LlmProbeInput,
  LlmSettingsInput,
  LlmSettingsView,
  MagiStreamEvent,
  OpeningBatch,
  OpeningRequest,
  ProviderProfileInfo,
  RuntimeFlags,
  SessionDraft,
  SessionRecord,
  SessionSummary,
  VoteBatch,
  VoteRequest,
} from '../types';

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function health(): Promise<string> {
  return invoke<string>('health');
}

export async function runOpening(
  request: OpeningRequest,
  onEvent: (event: MagiStreamEvent) => void,
): Promise<OpeningBatch> {
  const channel = new Channel<MagiStreamEvent>();
  channel.onmessage = onEvent;

  return invoke<OpeningBatch>('run_opening', { request, onEvent: channel });
}

export async function runDebate(
  request: DebateRequest,
  onEvent: (event: MagiStreamEvent) => void,
): Promise<DebateBatch> {
  const channel = new Channel<MagiStreamEvent>();
  channel.onmessage = onEvent;

  return invoke<DebateBatch>('run_debate', { request, onEvent: channel });
}

export async function castVotes(request: VoteRequest): Promise<VoteBatch> {
  return invoke<VoteBatch>('cast_votes', { request });
}

export async function getLlmSettings(): Promise<LlmSettingsView> {
  return invoke<LlmSettingsView>('get_llm_settings');
}

export async function saveLlmSettings(
  settings: LlmSettingsInput,
): Promise<LlmSettingsView> {
  return invoke<LlmSettingsView>('save_llm_settings', { settings });
}

export async function getLlmProviderCatalog(): Promise<ProviderProfileInfo[]> {
  return invoke<ProviderProfileInfo[]>('get_llm_provider_catalog');
}

export async function fetchLlmModels(
  input: LlmProbeInput,
): Promise<LlmModelList> {
  return invoke<LlmModelList>('fetch_llm_models', { input });
}

export async function testLlmConnection(
  input: LlmProbeInput,
): Promise<ConnectionTestResult> {
  return invoke<ConnectionTestResult>('test_llm_connection', { input });
}

export async function getRuntimeFlags(): Promise<RuntimeFlags> {
  return invoke<RuntimeFlags>('get_runtime_flags');
}

/** Only registered in debug builds. */
export async function setDemoMode(enabled: boolean): Promise<void> {
  return invoke<void>('set_demo_mode', { enabled });
}

export async function saveSession(
  session: SessionDraft,
): Promise<SessionSummary> {
  return invoke<SessionSummary>('save_session', { session });
}

export async function listSessions(): Promise<SessionSummary[]> {
  return invoke<SessionSummary[]>('list_sessions');
}

export async function getSession(id: string): Promise<SessionRecord> {
  return invoke<SessionRecord>('get_session', { id });
}

export async function deleteSession(id: string): Promise<void> {
  return invoke<void>('delete_session', { id });
}

export async function clearSessions(): Promise<void> {
  return invoke<void>('clear_sessions');
}
