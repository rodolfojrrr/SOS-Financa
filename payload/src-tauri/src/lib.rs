mod db;
mod sync;

use serde_json::Value;
use tauri::AppHandle;

#[tauri::command]
fn get_state(app: AppHandle) -> Result<Value, String> {
    db::state(&app)
}

#[tauri::command]
fn save_entity(app: AppHandle, entity_type: String, payload: Value) -> Result<String, String> {
    db::save(&app, &entity_type, &payload)
}

#[tauri::command]
fn archive_entity(app: AppHandle, entity_type: String, id: String) -> Result<(), String> {
    db::archive(&app, &entity_type, &id)
}

#[tauri::command]
fn make_backup(app: AppHandle) -> Result<String, String> {
    db::create_backup(&app)
}

#[tauri::command]
fn get_backups(app: AppHandle) -> Result<Vec<Value>, String> {
    db::list_backups(&app)
}

#[tauri::command]
fn restore_backup(app: AppHandle, name: String) -> Result<(), String> {
    db::restore_backup(&app, &name)
}

#[tauri::command]
fn get_database_info(app: AppHandle) -> Result<Value, String> {
    db::database_info(&app)
}

#[tauri::command]
fn get_sync_platform() -> Value {
    sync::platform()
}

#[tauri::command]
fn start_sync_server(app: AppHandle) -> Result<Value, String> {
    sync::start_server(&app)
}

#[tauri::command]
fn stop_sync_server() -> Result<(), String> {
    sync::stop_server();
    Ok(())
}

#[tauri::command]
async fn receive_sync_from_pc(app: AppHandle, host: String, port: u16, code: String) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || sync::receive_from_pc(&app, &host, port, &code))
        .await
        .map_err(|e| format!("Falha interna ao executar a sincronização fora da interface: {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            db::init(app.handle()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_entity,
            archive_entity,
            make_backup,
            get_backups,
            restore_backup,
            get_database_info,
            get_sync_platform,
            start_sync_server,
            stop_sync_server,
            receive_sync_from_pc
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o SOS Finança");
}
