use super::{DownloadOptions, DownloadProgress, Downloader, FormatInfo, VideoInfo};
use crate::database::models::AppSettings;
use crate::ytdlp::process::ProcessManager;
use std::path::PathBuf;
use std::sync::Arc;

pub struct YtDlpDownloader {
    process_manager: Arc<ProcessManager>,
    ytdlp_path: PathBuf,
    ffmpeg_path: Option<PathBuf>,
}

impl YtDlpDownloader {
    pub fn new(
        process_manager: Arc<ProcessManager>,
        ytdlp_path: PathBuf,
        ffmpeg_path: Option<PathBuf>,
    ) -> Self {
        Self {
            process_manager,
            ytdlp_path,
            ffmpeg_path,
        }
    }
}

#[async_trait::async_trait]
impl Downloader for YtDlpDownloader {
    async fn fetch_info(&self, url: &str) -> Result<VideoInfo, String> {
        // This needs to be async, but trait requires sync
        // We'll need to refactor this or use blocking
        Err("YtDlpDownloader::fetch_info requires async context".to_string())
    }

    async fn start_download(
        &self,
        _options: DownloadOptions,
        _progress_callback: Box<dyn Fn(DownloadProgress) + Send>,
    ) -> Result<u32, String> {
        // This needs to be async, but trait requires sync
        Err("YtDlpDownloader::start_download requires async context".to_string())
    }

    async fn cancel_download(&self, _pid: u32) -> Result<(), String> {
        Err("YtDlpDownloader::cancel_download requires async context".to_string())
    }

    async fn pause_download(&self, _pid: u32) -> Result<(), String> {
        Err("YtDlpDownloader::pause_download requires async context".to_string())
    }

    async fn resume_download(&self, _pid: u32) -> Result<(), String> {
        Err("YtDlpDownloader::resume_download requires async context".to_string())
    }
}
