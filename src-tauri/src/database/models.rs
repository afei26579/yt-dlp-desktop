use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub duration: Option<f64>,
    pub uploader: Option<String>,
    pub upload_date: Option<String>,
    pub description: Option<String>,
    pub webpage_url: String,
    pub formats: Vec<FormatInfo>,
    pub is_playlist: bool,
    pub playlist_count: Option<u32>,
    pub entries: Vec<PlaylistEntry>,
    pub available_subtitles: Vec<String>, // 可用的字幕语言列表
    pub has_subtitles: bool, // 是否有字幕
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub duration: Option<f64>,
    pub uploader: Option<String>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub format_note: Option<String>,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
    pub filesize_approx: Option<u64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub quality_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub total_size: Option<String>,
    pub downloaded_size: Option<String>,
    pub output_path: Option<String>,
    pub format_id: Option<String>,
    pub quality_label: String,
    pub audio_only: bool,
    pub download_subtitle: bool,
    pub subtitle_lang: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub playlist_title: Option<String>,
    pub playlist_index: Option<u32>,
    pub playlist_total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Fetching,
    Downloading,
    Queued,
    Paused,
    Merging,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub task_id: String,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub total_size: Option<String>,
    pub downloaded_size: Option<String>,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_path: String,
    pub max_concurrent: u32,
    pub filename_template: String,
    pub proxy_mode: ProxyMode,
    pub proxy_url: Option<String>,
    pub use_browser_cookie: bool,
    pub browser_type: String,
    pub cookie_file_path: Option<String>,
    pub auto_check_update: bool,
    pub minimize_to_tray: bool,
    pub clipboard_watch: bool,
    pub theme: String,
    pub language: String,
    pub extra_args: Option<String>,
    pub notify_on_complete: bool,
    pub notify_on_error: bool,
    pub speed_limit: Option<String>,
    pub download_thumbnail: bool,
    pub download_metadata: bool,
    pub audio_quality: String,
    pub douyin_api_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyMode {
    None,
    System,
    Custom,
}

impl Default for AppSettings {
    fn default() -> Self {
        let dp = dirs::video_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Videos"))
            .to_string_lossy()
            .to_string();
        Self {
            download_path: dp,
            max_concurrent: 3,
            filename_template: "%(title)s.%(ext)s".into(),
            proxy_mode: ProxyMode::System,
            proxy_url: None,
            use_browser_cookie: false,
            browser_type: "chrome".into(),
            cookie_file_path: None,
            auto_check_update: true,
            minimize_to_tray: true,
            clipboard_watch: false,
            theme: "system".into(),
            language: "zh-CN".into(),
            extra_args: None,
            notify_on_complete: true,
            notify_on_error: true,
            speed_limit: None,
            download_thumbnail: false,
            download_metadata: false,
            audio_quality: "0".into(),
            douyin_api_endpoint: Some("https://api.douyin.wtf".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieDiagnostic {
    /// 当前配置方式: "none", "file", "browser"
    pub method: String,
    /// 浏览器类型
    pub browser_type: String,
    /// Cookie 文件路径
    pub cookie_file_path: Option<String>,
    /// Cookie 文件是否存在
    pub cookie_file_exists: bool,
    /// Cookie 文件大小
    pub cookie_file_size: u64,
    /// 浏览器是否正在运行
    pub browser_running: bool,
    /// Cookie 数据库是否找到
    pub cookie_db_found: bool,
    /// Cookie 数据库路径检查结果
    pub cookie_db_paths: Vec<CookieDbInfo>,
    /// 实际测试结果
    pub test_result: Option<String>,
    /// 建议列表
    pub suggestions: Vec<String>,
}

/// Cookie 数据库信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieDbInfo {
    pub path: String,
    pub exists: bool,
    pub size: u64,
}