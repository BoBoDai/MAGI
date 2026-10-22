//! MAGI Tauri shell.
//!
//! This layer only wires adapters, registers commands, and opens the window.
//! Protocol logic lives in `magi-usecases`.

mod backend;
mod commands;
mod credentials;
mod sessions;
mod settings;

use backend::LlmBackend;
use magi_usecases::App;
use sessions::SessionStore;
use settings::SettingsManager;
use tauri::Manager;

/// Shared application state across IPC commands.
pub struct AppState {
    pub app: App<LlmBackend>,
    pub settings: SettingsManager,
    pub history: SessionStore,
}

#[cfg(debug_assertions)]
macro_rules! handlers {
    () => {
        tauri::generate_handler![
            commands::health,
            commands::run_opening,
            commands::run_debate,
            commands::cast_votes,
            commands::get_llm_settings,
            commands::save_llm_settings,
            commands::get_llm_provider_catalog,
            commands::fetch_llm_models,
            commands::test_llm_connection,
            commands::get_runtime_flags,
            commands::set_demo_mode,
            commands::save_session,
            commands::list_sessions,
            commands::get_session,
            commands::delete_session,
            commands::clear_sessions,
        ]
    };
}

#[cfg(not(debug_assertions))]
macro_rules! handlers {
    () => {
        tauri::generate_handler![
            commands::health,
            commands::run_opening,
            commands::run_debate,
            commands::cast_votes,
            commands::get_llm_settings,
            commands::save_llm_settings,
            commands::get_llm_provider_catalog,
            commands::fetch_llm_models,
            commands::test_llm_connection,
            commands::get_runtime_flags,
            commands::save_session,
            commands::list_sessions,
            commands::get_session,
            commands::delete_session,
            commands::clear_sessions,
        ]
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "magi_adapters=info,magi_app=info,tauri=warn".into()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|error| error.to_string())?;
            let settings = SettingsManager::load(config_dir);
            let llm = LlmBackend::new(settings.shared());

            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            let history = SessionStore::load(data_dir);

            app.manage(AppState {
                app: App::new(llm),
                settings,
                history,
            });

            Ok(())
        })
        .invoke_handler(handlers!())
        .run(tauri::generate_context!())
        .expect("MAGI failed to start");
}
