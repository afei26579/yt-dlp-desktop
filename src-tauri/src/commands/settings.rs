use tauri::State;
use crate::commands::download::AppState;
use crate::database::models::AppSettings;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(state.config.load())
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    // 同步剪贴板监听状态
    state.clipboard_watcher.set_enabled(settings.clipboard_watch).await;
    state.config.save(&settings).map_err(|e| format!("Failed to save: {}", e))
}