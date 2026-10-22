/**
 * Hand-written mirror of the Rust IPC types.
 *
 * Rust (`magi-core`, `magi-adapters`, `src-tauri`) is the source of truth.
 * Wire fields are snake_case; only top-level Tauri command arguments are
 * camelCase (`onEvent`).
 */

// ---------------------------------------------------------------------------
// Protocol domain
// ---------------------------------------------------------------------------

export type PersonaId = 'melchior' | 'balthasar' | 'casper';
export type Stage = 'opening' | 'debate' | 'vote';
export type Stance = 'approve' | 'reject' | 'abstain';

/** 对应 `magi_core::MagiStreamEvent`（serde 内部标签 `kind`） */
export type MagiStreamEvent =
  | {
      kind: 'stage_started';
      run_id: string;
      stage: Stage;
      personas: PersonaId[];
    }
  | { kind: 'persona_started'; run_id: string; stage: Stage; persona: PersonaId }
  | {
      kind: 'delta';
      run_id: string;
      stage: Stage;
      persona: PersonaId;
      text: string;
    }
  | {
      kind: 'persona_completed';
      run_id: string;
      stage: Stage;
      persona: PersonaId;
    }
  | {
      kind: 'persona_failed';
      run_id: string;
      stage: Stage;
      persona: PersonaId;
      message: string;
    }
  | { kind: 'stage_completed'; run_id: string; stage: Stage };

/** 对应 `magi_core::Opening` */
export interface Opening {
  persona: PersonaId;
  stance: Stance;
  confidence: number;
  summary: string;
  key_points: string[];
  risks: string[];
  model: string;
}

/** 对应 `magi_core::OpeningOutcome` */
export type OpeningOutcome =
  | { status: 'ok'; opening: Opening }
  | { status: 'failed'; persona: PersonaId; message: string };

/** 对应 `magi_core::DecisionStatus` */
export type DecisionStatus = 'info' | 'yes' | 'no' | 'conditional' | 'error';

/** 对应 `magi_core::OpeningBatch` */
export interface OpeningBatch {
  outcomes: OpeningOutcome[];
  prompt_version: string;
  decision: DecisionStatus;
}

export interface OpeningRequest {
  run_id: string;
  question: string;
  context: string;
}

/** 对应 `magi_core::DebateReply` */
export interface DebateReply {
  persona: PersonaId;
  text: string;
  model: string;
}

export type DebateOutcome =
  | { status: 'ok'; reply: DebateReply }
  | { status: 'failed'; persona: PersonaId; message: string };

export interface DebateBatch {
  outcomes: DebateOutcome[];
  prompt_version: string;
}

export interface DebateRequest {
  run_id: string;
  question: string;
  context: string;
  openings: OpeningBatch;
}

/** 对应 `magi_core::FinalVote` */
export interface FinalVote {
  persona: PersonaId;
  stance: Stance;
  confidence: number;
  reason: string;
  model: string;
  opening_stance: Stance | null;
  changed_from_opening: boolean | null;
}

export type VoteOutcome =
  | { status: 'ok'; vote: FinalVote }
  | { status: 'failed'; persona: PersonaId; message: string };

export interface VoteTally {
  approve: number;
  reject: number;
  abstain: number;
  failed: number;
}

/** 对应 `magi_core::MagiVerdict` */
export type MagiVerdict =
  | { kind: 'unanimous'; stance: Stance }
  | { kind: 'majority'; stance: Stance }
  | { kind: 'split' }
  | { kind: 'incomplete' };

export interface VoteBatch {
  outcomes: VoteOutcome[];
  tally: VoteTally;
  verdict: MagiVerdict;
  prompt_version: string;
}

export interface VoteRequest {
  question: string;
  context: string;
  openings: OpeningBatch;
  debate: DebateBatch | null;
}

// ---------------------------------------------------------------------------
// Session history
// ---------------------------------------------------------------------------

export type SessionSource = 'live' | 'mock';
export type SessionStatus = 'opening' | 'debate' | 'vote';

/** 对应 `magi_core::SessionDraft` */
export interface SessionDraft {
  id: string;
  question: string;
  context: string;
  openings: OpeningBatch | null;
  debate: DebateBatch | null;
  vote: VoteBatch | null;
}

/** 对应 `magi_core::SessionRecord` */
export interface SessionRecord extends SessionDraft {
  schema_version: number;
  created_at: string;
  updated_at: string;
  source: SessionSource;
}

/** 对应 `magi_core::SessionSummary` */
export interface SessionSummary {
  id: string;
  created_at: string;
  updated_at: string;
  question: string;
  status: SessionStatus;
  verdict: MagiVerdict | null;
  tally: VoteTally | null;
  opening_failures: number;
  debate_failures: number;
  source: SessionSource;
}

// ---------------------------------------------------------------------------
// Runtime flags
// ---------------------------------------------------------------------------

export interface RuntimeFlags {
  demo_available: boolean;
  demo_enabled: boolean;
  version: string;
}

// ---------------------------------------------------------------------------
// LLM settings (mirrors `magi_adapters` and `src-tauri::settings`)
// ---------------------------------------------------------------------------

export type ProviderProfileId = 'deepseek' | 'openai' | 'minimax' | 'custom';
export type MaxTokensField = 'max_tokens' | 'max_completion_tokens';
export type ThinkingFormat = 'thinking' | 'enable_thinking';
export type ThinkingMode = 'default' | 'disabled' | 'adaptive';

export interface LlmCompat {
  json_mode?: boolean | null;
  max_tokens_field?: MaxTokensField | null;
  thinking_mode?: ThinkingMode | null;
  thinking_format?: ThinkingFormat | null;
}

export interface EffectiveCapabilities {
  json_mode: boolean;
  max_tokens_field: MaxTokensField;
  thinking_mode: ThinkingMode;
  thinking_format: ThinkingFormat;
}

/** 对应 `magi_adapters::ProviderProfileInfo` */
export interface ProviderProfileInfo {
  id: ProviderProfileId;
  label: string;
  default_base_url: string;
  default_model: string;
  capabilities: EffectiveCapabilities;
  protocol: string;
}

/** 对应 `src-tauri::settings::LlmProviderView`（不含 key） */
export interface LlmProviderView {
  id: string;
  name: string;
  base_url: string;
  model: string;
  provider: ProviderProfileId;
  compat: LlmCompat;
  has_api_key: boolean;
}

/** 对应 `src-tauri::settings::LlmSettingsView` */
export interface LlmSettingsView {
  schema_version: number;
  providers: LlmProviderView[];
  active_id: string;
}

/** 对应 `src-tauri::settings::LlmProviderInput` */
export interface LlmProviderInput {
  id: string;
  name: string;
  base_url: string;
  model: string;
  provider: ProviderProfileId;
  compat: LlmCompat;
  api_key?: string | null;
  clear_api_key?: boolean;
}

export interface LlmSettingsInput {
  providers: LlmProviderInput[];
  active_id: string;
}

export interface LlmProbeInput {
  provider_id: string | null;
  base_url: string;
  model: string;
  provider: ProviderProfileId;
  compat: LlmCompat;
  api_key?: string | null;
}

export type LlmErrorKind =
  | 'missing_config'
  | 'bad_request'
  | 'unauthorized'
  | 'forbidden'
  | 'not_found'
  | 'rate_limited'
  | 'server'
  | 'timeout'
  | 'network'
  | 'unknown_model'
  | 'invalid_response';

export interface LlmEndpointError {
  kind: LlmErrorKind;
  message: string;
  status: number | null;
  request_id: string | null;
  provider_message: string | null;
}

export interface LlmModelList {
  models: string[];
  request_id: string | null;
}

export interface ConnectionTestResult {
  ok: boolean;
  latency_ms: number;
  model: string;
  request_id: string | null;
}
