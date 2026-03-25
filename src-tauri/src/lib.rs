#![allow(unused_imports)]

mod clipboard;
mod commands;
mod config;
mod database;
mod export;
mod queue;
mod updater;
mod ytdlp;

use std::sync::Arc;
use tauri::{
    Manager, Emitter,
    tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder},
};

use clipboard::ClipboardWatcher;
use commands::download::AppState;
use config::ConfigManager;
use database::Database;
use queue::DownloadQueue;
use ytdlp::process::ProcessManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .expect("Failed to get app data dir");
            log::info!("App data dir: {:?}", app_dir);

            let db = Arc::new(Database::new(app_dir.clone()).expect("Failed to init db"));
            let config = Arc::new(ConfigManager::new(app_dir.clone()));
            let process_manager = Arc::new(ProcessManager::new());
            let clipboard_watcher = Arc::new(ClipboardWatcher::new());

            let settings = config.load();
            let download_queue = Arc::new(DownloadQueue::new(settings.max_concurrent));

            // 启动队列消费
            queue::start_queue_worker(
                download_queue.clone(),
                process_manager.clone(),
                db.clone(),
                app.handle().clone(),
            );

            // 启动剪贴板监听
            if settings.clipboard_watch {
                let cw = clipboard_watcher.clone();
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    cw.set_enabled(true).await;
                });
                clipboard::start_clipboard_watcher(
                    clipboard_watcher.clone(),
                    app.handle().clone(),
                );
            } else {
                clipboard::start_clipboard_watcher(
                    clipboard_watcher.clone(),
                    app.handle().clone(),
                );
            }

            app.manage(AppState {
                db,
                process_manager,
                config,
                queue: download_queue,
                clipboard_watcher,
            });

            // 系统托盘
            setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::download::fetch_video_info,
            commands::download::start_download,
            commands::download::start_batch_download,
            commands::download::cancel_download,
            commands::download::get_download_history,
            commands::download::clear_history,
            commands::download::delete_history_item,
            commands::download::open_file_location,
            commands::download::start_batch_urls,        
            commands::download::pause_download,        
            commands::download::resume_download,   
            commands::download::check_ytdlp,
            commands::download::update_ytdlp,
            commands::download::diagnose_cookie,
            commands::download::get_queue_status,
            commands::download::update_max_concurrent,
            commands::download::set_clipboard_watch,
            commands::download::export_history,
            commands::download::import_urls,
            commands::download::export_settings_file,
            commands::download::import_settings_file,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    let settings = state.config.load();
                    if settings.minimize_to_tray {
                        api.prevent_close();
                        let _ = window.hide();
                        return;
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .separator()
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("YT-DLP Desktop")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}