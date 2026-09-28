mod db;

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
            get_database_info
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o SOS Finança");
}
