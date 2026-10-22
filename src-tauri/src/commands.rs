//! IPC commands. Every command is a thin shell over a usecase or a store.

use crate::AppState;
use crate::settings::{LlmProbeInput, LlmSettingsInput, LlmSettingsView};
use magi_adapters::{
    ConnectionTestResult, LlmEndpointError, LlmErrorKind, LlmModelList, ProviderProfileInfo,
    provider_catalog,
};
use magi_core::{
    DebateBatch, DebateRequest, MagiStreamEvent, OpeningBatch, OpeningRequest, SessionDraft,
    SessionRecord, SessionSummary, VoteBatch, VoteRequest,
};
use tauri::State;
use tauri::ipc::Channel;

type IpcResult<T> = std::result::Result<T, String>;

fn to_ipc(error: magi_core::Error) -> String {
    tracing::error!(error = %error, "usecase failed");
    error.to_string()
}

fn missing_config(message: String) -> LlmEndpointError {
    LlmEndpointError {
        kind: LlmErrorKind::MissingConfig,
        message,
        status: None,
        request_id: None,
        provider_message: None,
    }
}

#[tauri::command]
pub fn health() -> String {
    format!("MAGI {} ONLINE", env!("CARGO_PKG_VERSION"))
}

#[tauri::command]
pub async fn run_opening(
    state: State<'_, AppState>,
    request: OpeningRequest,
    on_event: Channel<MagiStreamEvent>,
) -> IpcResult<OpeningBatch> {
    let channel = on_event.clone();
    let emit = move |event: MagiStreamEvent| {
        if let Err(error) = channel.send(event) {
            tracing::warn!(%error, "MAGI channel send failed");
        }
    };

    state.app.run_opening(request, &emit).await.map_err(to_ipc)
}

#[tauri::command]
pub async fn run_debate(
    state: State<'_, AppState>,
    request: DebateRequest,
    on_event: Channel<MagiStreamEvent>,
) -> IpcResult<DebateBatch> {
    let channel = on_event.clone();
    let emit = move |event: MagiStreamEvent| {
        if let Err(error) = channel.send(event) {
            tracing::warn!(%error, "MAGI channel send failed");
        }
    };

    state.app.run_debate(request, &emit).await.map_err(to_ipc)
}

#[tauri::command]
pub async fn cast_votes(state: State<'_, AppState>, request: VoteRequest) -> IpcResult<VoteBatch> {
    state.app.cast_votes(request).await.map_err(to_ipc)
}

#[tauri::command]
pub fn get_llm_settings(state: State<'_, AppState>) -> IpcResult<LlmSettingsView> {
    Ok(state.settings.current_view())
}

#[tauri::command]
pub fn save_llm_settings(
    state: State<'_, AppState>,
    settings: LlmSettingsInput,
) -> IpcResult<LlmSettingsView> {
    let saved = state.settings.save(&settings)?;
    tracing::info!(
        providers = saved.providers.len(),
        active_id = %saved.active_id,
        "LLM settings updated"
    );
    Ok(saved)
}

#[tauri::command]
pub fn get_llm_provider_catalog() -> Vec<ProviderProfileInfo> {
    provider_catalog()
}

#[tauri::command]
pub async fn fetch_llm_models(
    state: State<'_, AppState>,
    input: LlmProbeInput,
) -> std::result::Result<LlmModelList, LlmEndpointError> {
    let config = state
        .settings
        .resolve_probe(&input)
        .map_err(missing_config)?;
    state.app.llm.fetch_models(&config).await
}

#[tauri::command]
pub async fn test_llm_connection(
    state: State<'_, AppState>,
    input: LlmProbeInput,
) -> std::result::Result<ConnectionTestResult, LlmEndpointError> {
    let config = state
        .settings
        .resolve_probe(&input)
        .map_err(missing_config)?;
    state.app.llm.test_connection(&config).await
}

#[derive(serde::Serialize)]
pub struct RuntimeFlags {
    pub demo_available: bool,
    pub demo_enabled: bool,
    pub version: String,
}

#[tauri::command]
pub fn get_runtime_flags(state: State<'_, AppState>) -> RuntimeFlags {
    RuntimeFlags {
        demo_available: state.app.llm.demo_available(),
        demo_enabled: state.app.llm.demo_enabled(),
        version: env!("CARGO_PKG_VERSION").into(),
    }
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn set_demo_mode(state: State<'_, AppState>, enabled: bool) -> IpcResult<()> {
    state.app.llm.set_demo_mode(enabled);
    tracing::info!(enabled, "demo mode changed");
    Ok(())
}

#[tauri::command]
pub fn save_session(
    state: State<'_, AppState>,
    session: SessionDraft,
) -> IpcResult<SessionSummary> {
    let source = state.app.llm.source();
    state.history.save(session, source).map_err(|error| {
        tracing::warn!(%error, "failed to save session");
        error
    })
}

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> IpcResult<Vec<SessionSummary>> {
    Ok(state.history.list())
}

#[tauri::command]
pub fn get_session(state: State<'_, AppState>, id: String) -> IpcResult<SessionRecord> {
    state
        .history
        .get(&id)
        .ok_or_else(|| "未找到该会话".to_string())
}

#[tauri::command]
pub fn delete_session(state: State<'_, AppState>, id: String) -> IpcResult<()> {
    state.history.delete(&id)
}

#[tauri::command]
pub fn clear_sessions(state: State<'_, AppState>) -> IpcResult<()> {
    state.history.clear()
}
