use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;
use chrono::Local;

use crate::database::Database;
use crate::database::models::*;
use crate::downloader::{detect_downloader_type, DownloaderType};
use crate::downloader::douyin::DouyinDownloader;
use crate::ytdlp::binary;
use crate::ytdlp::process::ProcessManager;
use crate::config::ConfigManager;
use crate::queue::{DownloadQueue, QueuedTask};
use crate::clipboard::ClipboardWatcher;
use crate::export;
use crate::updater;

pub struct AppState {
    pub db: Arc<Database>,
    pub process_manager: Arc<ProcessManager>,
    pub config: Arc<ConfigManager>,
    pub queue: Arc<DownloadQueue>,
    pub clipboard_watcher: Arc<ClipboardWatcher>,
}

#[tauri::command]
pub async fn fetch_video_info(url: String, state: State<'_, AppState>) -> Result<VideoInfo, String> {
    let downloader_type = detect_downloader_type(&url);

    match downloader_type {
        DownloaderType::Douyin => {
            log::info!("Detected Douyin/TikTok URL: {}", url);
            let config = state.config.load();

            // Try Douyin API first if endpoint is configured
            if let Some(ref api_endpoint) = config.douyin_api_endpoint {
                if !api_endpoint.is_empty() {
                    log::info!("Trying DouyinDownloader with API: {}", api_endpoint);
                    let downloader = DouyinDownloader::new(Some(api_endpoint.clone()));

                    use crate::downloader::Downloader;
                    match downloader.fetch_info(&url).await {
                        Ok(info) => return Ok(info),
                        Err(e) => {
                            log::warn!("DouyinDownloader failed: {}, falling back to yt-dlp", e);
                        }
                    }
                }
            }

            // Fallback to yt-dlp
            log::info!("Using yt-dlp as fallback for Douyin URL");
            let app_dir = state.config.app_dir();
            let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;
            state.process_manager.fetch_video_info(&ytdlp_path, &url, &config).await
        }
        DownloaderType::YtDlp => {
            log::info!("Using YtDlpDownloader for URL: {}", url);
            let config = state.config.load();
            let app_dir = state.config.app_dir();
            let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;
            state.process_manager.fetch_video_info(&ytdlp_path, &url, &config).await
        }
    }
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle, url: String, title: String, thumbnail: Option<String>,
    format_id: Option<String>, quality_label: String, audio_only: bool,
    download_subtitle: bool, subtitle_lang: Option<String>,
    playlist_title: Option<String>, playlist_index: Option<u32>, playlist_total: Option<u32>,
    state: State<'_, AppState>,
) -> Result<DownloadTask, String> {
    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;
    let ffmpeg_path = binary::get_ffmpeg_path(&app_dir);

    let task = DownloadTask {
        id: Uuid::new_v4().to_string(), url, title, thumbnail,
        status: DownloadStatus::Queued, progress: 0.0,
        speed: None, eta: None, total_size: None, downloaded_size: None,
        output_path: None, format_id, quality_label, audio_only,
        download_subtitle, subtitle_lang, error_message: None,
        created_at: Local::now().to_rfc3339(), completed_at: None,
        playlist_title, playlist_index, playlist_total,
    };
    state.db.insert_task(&task).map_err(|e| format!("DB: {}", e))?;

    let _ = app.emit("download-progress", &DownloadProgress {
        task_id: task.id.clone(), status: DownloadStatus::Queued, progress: 0.0,
        speed: None, eta: Some("排队中...".into()), total_size: None,
        downloaded_size: None, output_path: None, error_message: None,
    });

    state.queue.enqueue(QueuedTask {
        task: task.clone(), ytdlp_path, ffmpeg_path, settings: config, is_resume: false,
    }).await;

    Ok(task)
}

#[tauri::command]
pub async fn start_batch_download(
    app: AppHandle, entries: Vec<BatchDownloadEntry>,
    audio_only: bool, download_subtitle: bool, subtitle_lang: Option<String>,
    playlist_title: String, state: State<'_, AppState>,
) -> Result<Vec<DownloadTask>, String> {
    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;
    let ffmpeg_path = binary::get_ffmpeg_path(&app_dir);
    let total = entries.len() as u32;
    let mut tasks = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        let task = DownloadTask {
            id: Uuid::new_v4().to_string(), url: entry.url.clone(), title: entry.title.clone(),
            thumbnail: entry.thumbnail.clone(), status: DownloadStatus::Queued,
            progress: 0.0, speed: None, eta: None, total_size: None, downloaded_size: None,
            output_path: None, format_id: None,
            quality_label: if audio_only { "MP3 音频".into() } else { "最佳画质".into() },
            audio_only, download_subtitle, subtitle_lang: subtitle_lang.clone(),
            error_message: None, created_at: Local::now().to_rfc3339(), completed_at: None,
            playlist_title: Some(playlist_title.clone()),
            playlist_index: Some((i + 1) as u32), playlist_total: Some(total),
        };
        state.db.insert_task(&task).map_err(|e| format!("DB: {}", e))?;
        let _ = app.emit("download-progress", &DownloadProgress {
            task_id: task.id.clone(), status: DownloadStatus::Queued, progress: 0.0,
            speed: None, eta: Some(format!("排队中... ({}/{})", i + 1, total)),
            total_size: None, downloaded_size: None, output_path: None, error_message: None,
        });
        state.queue.enqueue(QueuedTask {
            task: task.clone(), ytdlp_path: ytdlp_path.clone(),
            ffmpeg_path: ffmpeg_path.clone(), settings: config.clone(), is_resume: false,
        }).await;
        tasks.push(task);
    }
    Ok(tasks)
}

/// ★ 批量 URL 下载（无需预先获取视频信息）
#[tauri::command]
pub async fn start_batch_urls(
    app: AppHandle, urls: Vec<String>, audio_only: bool,
    download_subtitle: bool, subtitle_lang: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DownloadTask>, String> {
    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;
    let ffmpeg_path = binary::get_ffmpeg_path(&app_dir);
    let total = urls.len() as u32;
    let mut tasks = Vec::new();

    for (i, url) in urls.iter().enumerate() {
        let short_title = shorten_url(url);
        let task = DownloadTask {
            id: Uuid::new_v4().to_string(), url: url.clone(), title: short_title,
            thumbnail: None, status: DownloadStatus::Queued,
            progress: 0.0, speed: None, eta: None, total_size: None, downloaded_size: None,
            output_path: None, format_id: None,
            quality_label: if audio_only { "MP3 音频".into() } else { "最佳画质".into() },
            audio_only, download_subtitle, subtitle_lang: subtitle_lang.clone(),
            error_message: None, created_at: Local::now().to_rfc3339(), completed_at: None,
            playlist_title: Some("批量下载".into()),
            playlist_index: Some((i + 1) as u32), playlist_total: Some(total),
        };
        state.db.insert_task(&task).map_err(|e| format!("DB: {}", e))?;
        let _ = app.emit("download-progress", &DownloadProgress {
            task_id: task.id.clone(), status: DownloadStatus::Queued, progress: 0.0,
            speed: None, eta: Some(format!("排队中... ({}/{})", i + 1, total)),
            total_size: None, downloaded_size: None, output_path: None, error_message: None,
        });
        state.queue.enqueue(QueuedTask {
            task: task.clone(), ytdlp_path: ytdlp_path.clone(),
            ffmpeg_path: ffmpeg_path.clone(), settings: config.clone(), is_resume: false,
        }).await;
        tasks.push(task);
    }
    log::info!("Batch URLs: {} tasks queued", tasks.len());
    Ok(tasks)
}

/// ★ 暂停下载
#[tauri::command]
pub async fn pause_download(task_id: String, state: State<'_, AppState>) -> Result<(), String> {
    log::info!("Pausing task: {}", task_id);
    state.process_manager.pause_download(&task_id).await?;
    state.db.update_task_status(&task_id,
        &serde_json::to_string(&DownloadStatus::Paused).unwrap(),
        None, None, None).map_err(|e| format!("DB: {}", e))
}

/// ★ 恢复下载
#[tauri::command]
pub async fn resume_download(
    app: AppHandle, task_id: String, state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Resuming task: {}", task_id);
    let queued_task = state.queue.take_paused(&task_id).await
        .ok_or("Task not found in paused list")?;

    let _ = app.emit("download-progress", &DownloadProgress {
        task_id: task_id.clone(), status: DownloadStatus::Queued, progress: 0.0,
        speed: None, eta: Some("恢复中...".into()), total_size: None,
        downloaded_size: None, output_path: None, error_message: None,
    });

    state.queue.enqueue(queued_task).await;
    state.db.update_task_status(&task_id,
        &serde_json::to_string(&DownloadStatus::Queued).unwrap(),
        None, None, None).map_err(|e| format!("DB: {}", e))
}

#[tauri::command]
pub async fn retry_download(
    app: AppHandle, task_id: String, state: State<'_, AppState>,
) -> Result<DownloadTask, String> {
    let history = state.db.get_history(10000, 0).map_err(|e| format!("DB: {}", e))?;
    let old_task = history.iter().find(|t| t.id == task_id)
        .ok_or("Task not found")?;

    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;
    let ffmpeg_path = binary::get_ffmpeg_path(&app_dir);

    // 创建新任务（新ID，相同参数）
    let new_task = DownloadTask {
        id: Uuid::new_v4().to_string(),
        url: old_task.url.clone(),
        title: old_task.title.clone(),
        thumbnail: old_task.thumbnail.clone(),
        status: DownloadStatus::Queued,
        progress: 0.0,
        speed: None, eta: None, total_size: None, downloaded_size: None,
        output_path: None,
        format_id: old_task.format_id.clone(),
        quality_label: old_task.quality_label.clone(),
        audio_only: old_task.audio_only,
        download_subtitle: old_task.download_subtitle,
        subtitle_lang: old_task.subtitle_lang.clone(),
        error_message: None,
        created_at: Local::now().to_rfc3339(),
        completed_at: None,
        playlist_title: old_task.playlist_title.clone(),
        playlist_index: old_task.playlist_index,
        playlist_total: old_task.playlist_total,
    };

    state.db.insert_task(&new_task).map_err(|e| format!("DB: {}", e))?;

    let _ = app.emit("download-progress", &DownloadProgress {
        task_id: new_task.id.clone(), status: DownloadStatus::Queued, progress: 0.0,
        speed: None, eta: Some("重试中...".into()), total_size: None,
        downloaded_size: None, output_path: None, error_message: None,
    });

    state.queue.enqueue(QueuedTask {
        task: new_task.clone(), ytdlp_path, ffmpeg_path,
        settings: config, is_resume: false,
    }).await;

    Ok(new_task)
}


#[tauri::command]
pub async fn cancel_download(
    task_id: String, state: State<'_, AppState>,
) -> Result<(), String> {
    // 从队列移除
    state.queue.remove_task(&task_id).await;
    // 取消正在执行的进程
    state.process_manager.cancel_download(&task_id).await?;
    // 更新数据库
    state.db.update_task_status(&task_id,
        &serde_json::to_string(&DownloadStatus::Cancelled).unwrap(),
        None, None, None).map_err(|e| format!("DB: {}", e))
}

#[derive(serde::Deserialize)]
pub struct BatchDownloadEntry {
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
}

#[tauri::command]
pub async fn get_download_history(limit: u32, offset: u32, state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    state.db.get_history(limit, offset).map_err(|e| format!("DB: {}", e))
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    log::info!("[DEBUG] clear_history command called");
    let result = state.db.clear_history().map_err(|e| format!("DB: {}", e));
    match &result {
        Ok(_) => log::info!("[DEBUG] clear_history completed successfully"),
        Err(e) => log::error!("[ERROR] clear_history failed: {}", e),
    }
    result
}

#[tauri::command]
pub async fn delete_history_item(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.db.delete_history(&id).map_err(|e| format!("DB: {}", e))
}

#[derive(serde::Serialize)]
pub struct QueueStatus { pub queue_size: usize, pub active_count: u32 }

#[tauri::command]
pub async fn get_queue_status(state: State<'_, AppState>) -> Result<QueueStatus, String> {
    Ok(QueueStatus { queue_size: state.queue.queue_size().await, active_count: state.queue.active_count_val().await })
}

#[tauri::command]
pub async fn update_max_concurrent(max: u32, state: State<'_, AppState>) -> Result<(), String> {
    state.queue.set_max_concurrent(max).await; Ok(())
}

#[tauri::command]
pub async fn open_file_location(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    #[cfg(target_os = "windows")]
    {
        if p.is_file() { std::process::Command::new("explorer").arg("/select,").arg(&path).spawn().map_err(|e| format!("{}", e))?; }
        else if p.is_dir() { std::process::Command::new("explorer").arg(&path).spawn().map_err(|e| format!("{}", e))?; }
        else if let Some(parent) = p.parent() { if parent.exists() { std::process::Command::new("explorer").arg(parent).spawn().map_err(|e| format!("{}", e))?; } else { return Err("路径不存在".into()); } }
        else { return Err("无效路径".into()); }
    }
    #[cfg(target_os = "macos")]
    { if p.is_file() { std::process::Command::new("open").arg("-R").arg(&path).spawn().map_err(|e| format!("{}", e))?; }
      else { std::process::Command::new("open").arg(if p.is_dir() { &path } else { p.parent().map(|pp| pp.to_str().unwrap_or("")).unwrap_or("") }).spawn().map_err(|e| format!("{}", e))?; } }
    #[cfg(target_os = "linux")]
    { let dir = if p.is_dir() { path.clone() } else { p.parent().map(|pp| pp.to_string_lossy().to_string()).unwrap_or(path.clone()) };
      std::process::Command::new("xdg-open").arg(&dir).spawn().map_err(|e| format!("{}", e))?; }
    Ok(())
}

#[tauri::command]
pub async fn check_ytdlp(state: State<'_, AppState>) -> Result<String, String> {
    let app_dir = state.config.app_dir();
    let p = binary::get_ytdlp_path(&app_dir)?;
    binary::get_ytdlp_version(&p)
}

async fn test_ytdlp_cookie(ytdlp_path: &std::path::PathBuf, config: &crate::database::models::AppSettings) -> String {
    use tokio::process::Command;
    use std::process::Stdio;

    // ★ 用 -v 模式 + 一个不存在的假 URL
    //   目的只是触发 cookie 读取流程，看 cookie 是否能成功加载
    //   不需要真正解析视频
    let mut cmd = Command::new(ytdlp_path);
    cmd.env("PYTHONIOENCODING", "utf-8")
       .env("PYTHONUTF8", "1");

    if let Some(ref cookie_file) = config.cookie_file_path {
        if !cookie_file.is_empty() && std::path::Path::new(cookie_file).exists() {
            cmd.arg("--cookies").arg(cookie_file);
        }
    } else if config.use_browser_cookie {
        cmd.arg("--cookies-from-browser").arg(&config.browser_type);
    }

    // ★ 关键改动：用 --dump-json 测试一个假 URL
    //   yt-dlp 会先加载 cookie，然后才尝试解析 URL
    //   我们只关心 cookie 加载阶段的输出
    cmd.args(["-v", "--skip-download", "--no-playlist", "https://www.example.com/test_cookie_check"])
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    match tokio::time::timeout(std::time::Duration::from_secs(15), cmd.output()).await {
        Ok(Ok(output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let all_output = format!("{}\n{}", stderr, stdout);

            let mut cookie_lines: Vec<String> = Vec::new();
            let mut error_lines: Vec<String> = Vec::new();
            let mut cookie_loaded = false;
            let mut cookie_count = 0u32;
            let mut cookie_source = String::new();
            let mut decrypt_error = false;

            for line in all_output.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                let lower = trimmed.to_lowercase();

                // ★ 检测 cookie 成功加载的标志
                if lower.contains("extracting cookies from") {
                    cookie_source = trimmed.to_string();
                    cookie_lines.push(trimmed.to_string());
                    log::info!("[DIAG-TEST] {}", trimmed);
                }

                if lower.contains("cookie version breakdown") || lower.contains("cookie") && lower.contains("v10") {
                    cookie_loaded = true;
                    cookie_lines.push(trimmed.to_string());
                    log::info!("[DIAG-TEST] {}", trimmed);

                    // 提取 cookie 数量：从 'v10': 1419 这样的格式中
                    if let Some(pos) = lower.find("v10'") {
                        let after = &trimmed[pos..];
                        let num_str: String = after.chars()
                            .skip_while(|c| !c.is_ascii_digit())
                            .take_while(|c| c.is_ascii_digit())
                            .collect();
                        cookie_count = num_str.parse().unwrap_or(0);
                    }
                }

                if (lower.contains("decrypt") || lower.contains("keyring"))
                    && !lower.contains("optional libraries")
                    && !lower.contains("cryptodome")
                {
                    decrypt_error = true;
                    cookie_lines.push(trimmed.to_string());
                    log::warn!("[DIAG-TEST][CRYPT] {}", trimmed);
                }

                if lower.contains("could not find") && lower.contains("cookie") {
                    cookie_lines.push(trimmed.to_string());
                    log::warn!("[DIAG-TEST] {}", trimmed);
                }

                if lower.starts_with("error") {
                    error_lines.push(trimmed.to_string());
                }
            }

            // ★ 判断结果：cookie 加载成功 ≠ URL 解析成功
            //   只要看到 "Extracting cookies from" + "cookie version breakdown" 就说明 cookie 读取没问题
            //   URL 报错 "Unsupported URL" 是正常的（我们用的假 URL）

            if cookie_loaded && cookie_count > 0 && !decrypt_error {
                format!(
                    "✅ Cookie 读取成功！\n\
                     📂 来源: {}\n\
                     🍪 已加载 {} 个 Cookie\n\
                     \n\
                     Cookie 配置正常，可以正常下载需要登录的视频。",
                    cookie_source
                        .replace("[debug] Extracting cookies from: ", "")
                        .replace("\"", ""),
                    cookie_count
                )
            } else if decrypt_error {
                let detail = cookie_lines.iter()
                    .filter(|l| l.to_lowercase().contains("decrypt") || l.to_lowercase().contains("crypt"))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "❌ Cookie 解密失败\n\
                     \n\
                     {}\n\
                     \n\
                     Chrome v127+ 使用了新的加密方式，yt-dlp 可能无法解密。",
                    detail
                )
            } else if !cookie_source.is_empty() && cookie_count == 0 {
                format!(
                    "⚠️ 找到 Cookie 数据库但没有有效 Cookie\n\
                     📂 来源: {}\n\
                     \n\
                     请确认已在浏览器中登录目标网站。",
                    cookie_source
                        .replace("[debug] Extracting cookies from: ", "")
                        .replace("\"", ""),
                )
            } else {
                // 没有任何 cookie 相关输出
                let relevant = if !cookie_lines.is_empty() {
                    cookie_lines.join("\n")
                } else if !error_lines.is_empty() {
                    // 过滤掉 "Unsupported URL" 这类与 cookie 无关的错误
                    let real_errors: Vec<String> = error_lines.iter()
                        .filter(|l| !l.contains("Unsupported URL"))
                        .cloned()
                        .collect();
                    if real_errors.is_empty() {
                        "未检测到 Cookie 加载信息".into()
                    } else {
                        real_errors.join("\n")
                    }
                } else {
                    "未检测到任何 Cookie 相关输出".into()
                };
                format!("❌ Cookie 读取失败\n\n{}", relevant)
            }
        }
        Ok(Err(e)) => {
            format!("❌ 无法启动 yt-dlp: {}", e)
        }
        Err(_) => {
            "❌ 测试超时（15秒），可能网络无法连通或 yt-dlp 卡住".into()
        }
    }
}


#[tauri::command]
pub async fn diagnose_cookie(state: State<'_, AppState>) -> Result<CookieDiagnostic, String> {
    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;

    log::info!("=== Starting Cookie Diagnosis ===");

    let mut diagnostic = CookieDiagnostic {
        method: "none".into(),
        browser_type: config.browser_type.clone(),
        cookie_file_path: config.cookie_file_path.clone(),
        cookie_file_exists: false,
        cookie_file_size: 0,
        browser_running: false,
        cookie_db_found: false,
        cookie_db_paths: Vec::new(),
        test_result: None,
        suggestions: Vec::new(),
    };

    // ===== 1. Cookie 文件检测 =====
    if let Some(ref path) = config.cookie_file_path {
        if !path.is_empty() {
            diagnostic.method = "file".into();
            let p = std::path::Path::new(path);
            diagnostic.cookie_file_exists = p.exists();
            diagnostic.cookie_file_size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);

            if !diagnostic.cookie_file_exists {
                diagnostic.suggestions.push("Cookie 文件不存在，请重新选择文件".into());
            } else if diagnostic.cookie_file_size == 0 {
                diagnostic.suggestions.push("Cookie 文件为空，请重新导出".into());
            } else {
                if let Ok(content) = std::fs::read_to_string(p) {
                    let has_entries = content.lines().any(|l| {
                        let parts: Vec<&str> = l.split('\t').collect();
                        parts.len() >= 7 && !l.starts_with('#')
                    });
                    if !has_entries {
                        diagnostic.suggestions.push("Cookie 文件格式可能不正确，请使用 Netscape 格式（.txt）".into());
                    }
                    let has_douyin = content.contains("douyin.com") || content.contains(".douyin.com");
                    let has_tiktok = content.contains("tiktok.com") || content.contains(".tiktok.com");
                    if !has_douyin && !has_tiktok {
                        diagnostic.suggestions.push("Cookie 文件中未找到抖音/TikTok 的 Cookie，请确认已在浏览器中登录后再导出".into());
                    }
                }
            }
        }
    }

    // ===== 2. 浏览器 Cookie 检测 =====
    if config.use_browser_cookie {
        diagnostic.method = "browser".into();

        #[cfg(target_os = "windows")]
        {
            let process_name = match config.browser_type.as_str() {
                "chrome" => "chrome.exe",
                "edge" => "msedge.exe",
                "firefox" => "firefox.exe",
                _ => "",
            };

            if !process_name.is_empty() {
                if let Ok(output) = std::process::Command::new("tasklist")
                    .args(["/FI", &format!("IMAGENAME eq {}", process_name), "/NH"])
                    .output()
                {
                    diagnostic.browser_running = String::from_utf8_lossy(&output.stdout).contains(process_name);
                }
            }

            let user_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let db_paths = match config.browser_type.as_str() {
                "chrome" => vec![
                    format!("{}\\Google\\Chrome\\User Data\\Default\\Network\\Cookies", user_data),
                    format!("{}\\Google\\Chrome\\User Data\\Default\\Cookies", user_data),
                ],
                "edge" => vec![
                    format!("{}\\Microsoft\\Edge\\User Data\\Default\\Network\\Cookies", user_data),
                    format!("{}\\Microsoft\\Edge\\User Data\\Default\\Cookies", user_data),
                ],
                _ => vec![],
            };

            for db_path in &db_paths {
                let exists = std::path::Path::new(db_path).exists();
                let size = if exists { std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0) } else { 0 };
                diagnostic.cookie_db_paths.push(CookieDbInfo { path: db_path.clone(), exists, size });
                if exists && size > 0 { diagnostic.cookie_db_found = true; }
            }

            if diagnostic.browser_running {
                diagnostic.suggestions.push(format!(
                    "{} 浏览器正在运行，可能导致 Cookie 数据库被锁定。建议关闭浏览器后重试。",
                    config.browser_type
                ));
            }
            if !diagnostic.cookie_db_found {
                diagnostic.suggestions.push(format!(
                    "未找到 {} 的 Cookie 数据库文件。请确认浏览器已安装并登录过目标网站。",
                    config.browser_type
                ));
            }
        }
    }

    // ===== 3. 调用 test_ytdlp_cookie 做实际测试 =====
    let test_result = test_ytdlp_cookie(&ytdlp_path, &config).await;
    log::info!("[DIAG] Test result: {}", test_result);
    diagnostic.test_result = Some(test_result.clone());

    // ===== 4. 根据测试结果追加建议 =====
    let test_lower = test_result.to_lowercase();

    if test_lower.contains("✅") {
        // Cookie 读取成功，不需要额外建议
        log::info!("[DIAG] Cookie test PASSED");
    } else {
        if test_lower.contains("decrypt") || test_lower.contains("crypt") || test_lower.contains("app-bound") {
            diagnostic.suggestions.push(
                "Chrome v127+ 使用了 App-Bound Encryption 加密 Cookie，yt-dlp 无法解密。\n\
                 强烈推荐改用 Cookie 文件方式：安装浏览器扩展「Get cookies.txt LOCALLY」导出。".into()
            );
        }
        if test_lower.contains("permission") || test_lower.contains("locked") || test_lower.contains("access") {
            diagnostic.suggestions.push("Cookie 数据库被锁定或无权限。请关闭浏览器后重试。".into());
        }
        if test_lower.contains("could not find") || test_lower.contains("no suitable") {
            diagnostic.suggestions.push("yt-dlp 找不到浏览器 Cookie 存储位置。可能浏览器使用了非默认 Profile。".into());
        }
        if test_lower.contains("没有有效") {
            diagnostic.suggestions.push("Cookie 数据库中没有有效的 Cookie。请确认已在浏览器中登录目标网站后重试。".into());
        }
    }

    if diagnostic.method == "none" {
        diagnostic.suggestions.push("当前未配置任何 Cookie。如需下载需要登录的视频，请配置 Cookie。".into());
        diagnostic.suggestions.push("推荐方式：安装浏览器扩展「Get cookies.txt LOCALLY」导出 Cookie 文件。".into());
    }

    log::info!("=== Cookie Diagnosis Complete ===");
    Ok(diagnostic)
}

#[tauri::command]
pub async fn update_ytdlp(state: State<'_, AppState>) -> Result<String, String> {
    let app_dir = state.config.app_dir();
    let p = binary::get_ytdlp_path(&app_dir)?;
    updater::update_ytdlp(&p).await
}

#[tauri::command]
pub async fn set_clipboard_watch(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.clipboard_watcher.set_enabled(enabled).await; Ok(())
}

#[tauri::command]
pub async fn export_history(format: String, path: String, state: State<'_, AppState>) -> Result<(), String> {
    let tasks = state.db.get_history(10000, 0).map_err(|e| format!("DB: {}", e))?;
    match format.as_str() { "json" => export::export_history_json(&tasks, &path), "csv" => export::export_history_csv(&tasks, &path), _ => Err("不支持".into()) }
}

#[tauri::command]
pub async fn import_urls(path: String) -> Result<Vec<String>, String> { export::import_urls_from_file(&path) }

#[tauri::command]
pub async fn export_settings_file(path: String, state: State<'_, AppState>) -> Result<(), String> {
    export::export_settings(&state.config.load(), &path)
}

#[tauri::command]
pub async fn import_settings_file(path: String, state: State<'_, AppState>) -> Result<AppSettings, String> {
    let s = export::import_settings(&path)?;
    state.config.save(&s).map_err(|e| format!("{}", e))?;
    Ok(s)
}

fn shorten_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let host = parsed.host_str().unwrap_or("video");
        let path = parsed.path();
        let short_path = if path.len() > 30 { &path[..30] } else { path };
        format!("{}{}", host, short_path)
    } else {
        url.chars().take(60).collect()
    }
}