pub mod douyin;
pub mod ytdlp;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync {
    async fn fetch_info(&self, url: &str) -> Result<VideoInfo, String>;
    async fn start_download(&self, options: DownloadOptions, progress_callback: Box<dyn Fn(DownloadProgress) + Send>) -> Result<u32, String>;
    async fn cancel_download(&self, pid: u32) -> Result<(), String>;
    async fn pause_download(&self, pid: u32) -> Result<(), String>;
    async fn resume_download(&self, pid: u32) -> Result<(), String>;
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
