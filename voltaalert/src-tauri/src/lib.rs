use std::sync::Arc;

use log::{debug, error, info};
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use voltaalert_core::alarm::AlarmDescriptor;
use voltaalert_core::alarm::manager::AlarmManager;
use voltaalert_core::zmq::{ZeromqClient, ZeromqConfig};
use voltaalert_core::repo::{AlertData, AlarmData};
use voltaalert_core::repo::sqlite::alert::SqliteAlertRepo;
use voltaalert_core::repo::sqlite::alarm::SqliteAlarmRepo;
use voltaalert_core::service::AlertService;

struct AppService(AlertService);
unsafe impl Send for AppService {}

type AppState = Arc<Mutex<AppService>>;

// ******************** Alerts

#[tauri::command]
async fn list_alerts(
    state: tauri::State<'_, AppState>,
    last_fetched_id: Option<String>,
) -> Result<Vec<AlertData>, String> {
    debug!("Command list_alerts: {:#?}", last_fetched_id);
    state.lock().await.0.list_alerts(last_fetched_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_alert(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    debug!("Command delete_alert: {:#?}", id);
    state.lock().await.0.delete_alert(id).map_err(|e| e.to_string())
}

// ******************** Alarms

#[tauri::command]
async fn list_alarms(
    state: tauri::State<'_, AppState>,
    last_fetched_id: Option<String>,
) -> Result<Vec<AlarmData>, String> {
    debug!("Command list_alarms: {:#?}", last_fetched_id);
    state.lock().await.0.list_alarms(last_fetched_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_alarm(
    state: tauri::State<'_, AppState>,
    description: String,
    kind: String,
    config: serde_json::Value,
) -> Result<AlarmData, String> {
    let configs = serde_json::to_vec(&config)
        .map_err(|e| format!("Invalid config: {}", e))?;
    let descriptor = AlarmDescriptor::new(description, kind, configs);
    state.lock().await.0.create_alarm(descriptor).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_alarm(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state.lock().await.0.delete_alarm(id).map_err(|e| e.to_string())
}

// ************************* Setup

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {

            // Prepate repo and service
            let alert_repo = SqliteAlertRepo::from_path("volta.db").expect("Failed to open SQLite database");
            let alarm_repo = SqliteAlarmRepo::from_path("volta.db").expect("Failed to open SQLite database");
            let alarm_service = AlarmManager::new(Box::new(alarm_repo)).expect("Failed to init AlarmService");
            let (service, mut alert_rx) = AlertService::new(Box::new(alert_repo), alarm_service);

            let state: AppState = Arc::new(Mutex::new(AppService(service)));
            app.manage(state.clone());

            let app_handle = app.handle().clone();

            // Worker for frontend alerting
            tauri::async_runtime::spawn(async move {
                while let Some(alert_data) = alert_rx.recv().await {
                    app_handle.emit("alert", alert_data).ok();
                }
            });

            // Worker for service alerting
            tauri::async_runtime::spawn(async move {
                let mut client = loop {
                    match ZeromqClient::connect(ZeromqConfig::default()).await {
                        Ok(c) => break c,
                        Err(e) => {
                            error!("ZMQ connect: {e}, retrying in 2s");
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        }
                    }
                };
                info!("ZMQ connected");
                loop {
                    let data = client.recv().await;
                    debug!("Got data ! {:#?}", data);
                    state.lock().await.0.evaluate_received_data(data).ok();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            //update_alert,
            delete_alert,
            list_alerts,
            list_alarms,
            create_alarm,
            delete_alarm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
