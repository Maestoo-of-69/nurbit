//! Nurbit desktop application: Tauri setup and the commands the UI may call.
//!
//! A *command* is a Rust function the interface calls with `invoke("name")`.
//! Commands stay thin: medical rules live in `medical-core`, storage in
//! `local-store`. See `commands.rs`, `specialist.rs` and `sync.rs`.

mod commands;
mod specialist;
mod sync;

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use nurbit_local_store::LocalStore;
use nurbit_local_store::seed::{
    seed_demo_crew_if_empty, seed_demo_measurements_if_empty, seed_second_specialist_if_missing,
};
use tauri::Manager;

/// Shared by all commands (Tauri hands it to any command that asks for it).
pub struct AppState {
    pub store: LocalStore,
    pub device_id: String,
    /// Serializes sync attempts (manual button and background timer).
    pub sync_lock: tauri::async_runtime::Mutex<()>,
    /// Result of the latest sync attempt, shown on the Sync screen.
    pub last_sync: Mutex<Option<sync::SyncReport>>,
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Opens (or creates) the local database in the user's app-data folder,
/// e.g. C:\Users\<name>\AppData\Roaming\com.nurbit.desktop\nurbit.db
fn open_state(app: &tauri::App) -> Result<AppState, Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    tauri::async_runtime::block_on(async {
        let store = LocalStore::open(&dir.join("nurbit.db")).await?;
        seed_demo_crew_if_empty(&store, now_ms()).await?;
        seed_demo_measurements_if_empty(&store, now_ms()).await?;
        seed_second_specialist_if_missing(&store, now_ms()).await?;
        let device_id = store.device_id().await?;
        Ok(AppState {
            store,
            device_id,
            sync_lock: tauri::async_runtime::Mutex::new(()),
            last_sync: Mutex::new(None),
        })
    })
}

/// Tries to sync every `BACKGROUND_INTERVAL`; does nothing until a server is configured.
fn start_background_sync(handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(sync::BACKGROUND_INTERVAL).await;
            let state = handle.state::<AppState>();
            sync::run_sync(&state).await;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = open_state(app)?;
            app.manage(state);
            start_background_sync(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::daily_questions,
            commands::risk_modules,
            commands::list_crew,
            commands::plan_follow_ups,
            commands::save_checkin,
            commands::list_checkins,
            commands::sync_status,
            commands::save_sync_settings,
            commands::sync_now,
            commands::assistant_providers,
            commands::ask_assistant,
            commands::member_trends,
            commands::risk_assessments,
            specialist::crew_overview,
            specialist::member_record,
            specialist::add_note,
            specialist::annotate_answer,
            specialist::review_baseline,
            specialist::record_decision,
            specialist::review_decision,
            specialist::pending_reviews,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Nurbit application");
}
