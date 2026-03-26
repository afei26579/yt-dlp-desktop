pub mod douyin;
pub mod ytdlp;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub duration: Option<u64>,
    pub uploader: Option<String>,
    pub formats: Vec<FormatInfo>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub quality_label: String,
    pub ext: String,
    pub filesize: Option<u64>,
    pub has_audio: bool,
    pub has_video: bool,
}

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub url: String,
    pub format_id: Option<String>,
    pub audio_only: bool,
    pub download_subtitle: bool,
    pub subtitle_lang: Option<String>,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub url: String,
    pub progress: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub total_size: Option<String>,
    pub downloaded_size: Option<String>,
    pub status: String,
}

pub trait Downloader: Send + Sync {
    fn fetch_info(&self, url: &str) -> Result<VideoInfo, String>;
    fn start_download(&self, options: DownloadOptions, progress_callback: Box<dyn Fn(DownloadProgress) + Send>) -> Result<u32, String>;
    fn cancel_download(&self, pid: u32) -> Result<(), String>;
    fn pause_download(&self, pid: u32) -> Result<(), String>;
    fn resume_download(&self, pid: u32) -> Result<(), String>;
}

pub fn detect_downloader_type(url: &str) -> DownloaderType {
    let url_lower = url.to_lowercase();

    if url_lower.contains("douyin.com") || url_lower.contains("tiktok.com") {
        DownloaderType::Douyin
    } else {
        DownloaderType::YtDlp
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloaderType {
    YtDlp,
    Douyin,
}
