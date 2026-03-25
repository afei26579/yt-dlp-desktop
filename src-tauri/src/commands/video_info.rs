use tauri::State;
use crate::commands::download::AppState;
use crate::database::models::VideoInfo;
use crate::ytdlp::binary;

/// 获取视频信息（独立命令，可用于预览/批量解析）
#[tauri::command]
pub async fn fetch_video_info_detail(
    url: String,
    state: State<'_, AppState>,
) -> Result<VideoInfo, String> {
    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;

    state.process_manager
        .fetch_video_info(&ytdlp_path, &url, &config)
        .await
}

/// 批量获取视频信息（用于播放列表展开）
#[tauri::command]
pub async fn fetch_playlist_info(
    url: String,
    state: State<'_, AppState>,
) -> Result<Vec<VideoInfo>, String> {
    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;

    // 先获取主信息判断是否为播放列表
    let info = state.process_manager
        .fetch_video_info(&ytdlp_path, &url, &config)
        .await?;

    if info.is_playlist {
        // 对于播放列表，用 --flat-playlist 获取所有条目
        // 这里简化处理，返回单个信息
        // 完整实现需要解析 yt-dlp 的 playlist entries
        Ok(vec![info])
    } else {
        Ok(vec![info])
    }
}

/// 获取支持的格式列表（详细版）
#[tauri::command]
pub async fn list_formats(
    url: String,
    state: State<'_, AppState>,
) -> Result<VideoInfo, String> {
    let config = state.config.load();
    let app_dir = state.config.app_dir();
    let ytdlp_path = binary::get_ytdlp_path(&app_dir)?;

    state.process_manager
        .fetch_video_info(&ytdlp_path, &url, &config)
        .await
}