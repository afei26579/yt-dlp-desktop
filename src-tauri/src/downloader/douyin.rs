use super::{DownloadOptions, DownloadProgress, Downloader, FormatInfo, VideoInfo};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

static NEXT_PID: AtomicU32 = AtomicU32::new(10000);

#[derive(Debug, Clone)]
pub struct DouyinDownloader {
    client: Client,
    api_base: String,
}

#[derive(Debug, Deserialize)]
struct DouyinApiResponse {
    #[serde(flatten)]
    response: DouyinApiResponseType,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DouyinApiResponseType {
    Success {
        code: i32,
        message: String,
        data: Option<DouyinVideoData>,
    },
    Error {
        detail: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct DouyinVideoData {
    #[serde(rename = "aweme_id")]
    aweme_id: Option<String>,
    desc: Option<String>,
    #[serde(rename = "create_time")]
    create_time: Option<u64>,
    author: Option<DouyinAuthor>,
    music: Option<DouyinMusic>,
    statistics: Option<DouyinStatistics>,
    video: Option<DouyinVideo>,
    images: Option<Vec<DouyinImage>>,
}

#[derive(Debug, Deserialize)]
struct DouyinAuthor {
    nickname: Option<String>,
    uid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DouyinMusic {
    title: Option<String>,
    author: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DouyinStatistics {
    #[serde(rename = "digg_count")]
    digg_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct DouyinVideo {
    #[serde(rename = "play_addr")]
    play_addr: Option<DouyinPlayAddr>,
    duration: Option<u64>,
    cover: Option<DouyinCover>,
}

#[derive(Debug, Deserialize)]
struct DouyinPlayAddr {
    url_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DouyinCover {
    url_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DouyinImage {
    url_list: Option<Vec<String>>,
}

impl DouyinDownloader {
    pub fn new(api_base: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            api_base: api_base.unwrap_or_else(|| "https://api.douyin.wtf".to_string()),
        }
    }

    fn parse_video_data(&self, url: &str, data: DouyinVideoData) -> Result<VideoInfo, String> {
        let video_id = data
            .aweme_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let title = data
            .desc
            .clone()
            .unwrap_or_else(|| "Untitled".to_string());

        let uploader = data
            .author
            .as_ref()
            .and_then(|a| a.nickname.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let thumbnail = data
            .video
            .as_ref()
            .and_then(|v| v.cover.as_ref())
            .and_then(|c| c.url_list.as_ref())
            .and_then(|list: &Vec<String>| list.first())
            .cloned();

        let duration = data.video.as_ref().and_then(|v| v.duration);

        let mut formats = Vec::new();

        // Add video format if available
        if let Some(video) = &data.video {
            if let Some(play_addr) = &video.play_addr {
                if let Some(url_list) = &play_addr.url_list {
                    if let Some(_video_url) = url_list.first() {
                        formats.push(FormatInfo {
                            format_id: "video".to_string(),
                            format_note: Some("原画".to_string()),
                            ext: "mp4".to_string(),
                            resolution: None,
                            filesize: None,
                            filesize_approx: None,
                            vcodec: Some("h264".to_string()),
                            acodec: Some("aac".to_string()),
                            quality_label: "原画".to_string(),
                        });
                    }
                }
            }
        }

        // Add audio-only format
        formats.push(FormatInfo {
            format_id: "audio".to_string(),
            format_note: Some("音频".to_string()),
            ext: "m4a".to_string(),
            resolution: None,
            filesize: None,
            filesize_approx: None,
            vcodec: None,
            acodec: Some("aac".to_string()),
            quality_label: "音频".to_string(),
        });

        Ok(VideoInfo {
            id: video_id.clone(),
            title: title.clone(),
            url: url.to_string(),
            thumbnail,
            duration: duration.map(|d| d as f64),
            uploader: Some(uploader),
            upload_date: None,
            description: data.desc.clone(),
            webpage_url: url.to_string(),
            formats,
            is_playlist: false,
            playlist_count: None,
            entries: Vec::new(),
            available_subtitles: Vec::new(),
            has_subtitles: false,
        })
    }
}

#[async_trait::async_trait]
impl Downloader for DouyinDownloader {
    async fn fetch_info(&self, url: &str) -> Result<VideoInfo, String> {
        // Try hybrid endpoint first (supports both URL and ID)
        let api_url = format!("{}/api/hybrid/video_data", self.api_base);

        log::info!("Fetching Douyin video info from: {}", api_url);

        let response = self
            .client
            .get(&api_url)
            .query(&[("url", url)])
            .send()
            .await
            .map_err(|e| format!("Failed to fetch video info: {}", e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        log::info!("API response status: {}, body: {}", status, &response_text[..response_text.len().min(500)]);

        let api_response: DouyinApiResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse API response: {} (response: {})", e, &response_text[..response_text.len().min(200)]))?;

        match api_response.response {
            DouyinApiResponseType::Success { code, message, data } => {
                if code != 200 {
                    return Err(format!("API error: {}", message));
                }

                let data = data.ok_or_else(|| "No video data in response".to_string())?;

                self.parse_video_data(url, data)
            }
            DouyinApiResponseType::Error { detail } => {
                Err(format!("API error: {}", detail))
            }
        }
    }

    async fn start_download(
        &self,
        options: DownloadOptions,
        progress_callback: Box<dyn Fn(DownloadProgress) + Send + Sync>,
    ) -> Result<u32, String> {
        let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);

        let api_url = format!("{}/api/hybrid/video_data", self.api_base);
        let client = self.client.clone();
        let url = options.url.clone();
        let output_dir = options.output_dir.clone();
        let audio_only = options.audio_only;

        let progress_callback = Arc::new(progress_callback);

        tokio::spawn(async move {
            let result = async {
                // Fetch video info first
                let response = client
                    .get(&api_url)
                    .query(&[("url", url.as_str())])
                    .send()
                    .await
                    .map_err(|e| format!("Failed to fetch video info: {}", e))?;

                let response_text = response
                    .text()
                    .await
                    .map_err(|e| format!("Failed to read response: {}", e))?;

                let api_response: DouyinApiResponse = serde_json::from_str(&response_text)
                    .map_err(|e| format!("Failed to parse API response: {}", e))?;

                let data = match api_response.response {
                    DouyinApiResponseType::Success { code, message, data } => {
                        if code != 200 {
                            return Err(format!("API error: {}", message));
                        }
                        data.ok_or_else(|| "No video data in response".to_string())?
                    }
                    DouyinApiResponseType::Error { detail } => {
                        return Err(format!("API error: {}", detail));
                    }
                };

                let title = data
                    .desc
                    .clone()
                    .unwrap_or_else(|| "video".to_string())
                    .chars()
                    .filter(|c| !r#"\/:*?"<>|"#.contains(*c))
                    .collect::<String>();

                let download_url = if audio_only {
                    // For audio, we still download video and extract audio later
                    data.video
                        .as_ref()
                        .and_then(|v| v.play_addr.as_ref())
                        .and_then(|p| p.url_list.as_ref())
                        .and_then(|list: &Vec<String>| list.first())
                        .ok_or_else(|| "No video URL found".to_string())?
                } else {
                    data.video
                        .as_ref()
                        .and_then(|v| v.play_addr.as_ref())
                        .and_then(|p| p.url_list.as_ref())
                        .and_then(|list: &Vec<String>| list.first())
                        .ok_or_else(|| "No video URL found".to_string())?
                };

                let ext = if audio_only { "m4a" } else { "mp4" };
                let output_path = output_dir.join(format!("{}.{}", title, ext));

                progress_callback(DownloadProgress {
                    task_id: url.clone(),
                    status: crate::database::models::DownloadStatus::Downloading,
                    progress: 0.0,
                    speed: None,
                    eta: None,
                    total_size: None,
                    downloaded_size: None,
                    output_path: None,
                    error_message: None,
                });

                // Download the file
                let mut response = client
                    .get(download_url)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to download: {}", e))?;

                let total_size = response.content_length();
                let mut file = File::create(&output_path)
                    .await
                    .map_err(|e| format!("Failed to create file: {}", e))?;

                let mut downloaded: u64 = 0;

                use tokio::io::AsyncReadExt;
                let mut buffer = [0u8; 8192];

                while let Ok(n) = response.chunk().await {
                    if let Some(chunk) = n {
                        if chunk.is_empty() {
                            break;
                        }

                        file.write_all(&chunk)
                            .await
                            .map_err(|e| format!("Write error: {}", e))?;

                        downloaded += chunk.len() as u64;

                        if let Some(total) = total_size {
                            let progress = (downloaded as f64 / total as f64) * 100.0;
                            progress_callback(DownloadProgress {
                                task_id: url.clone(),
                                status: crate::database::models::DownloadStatus::Downloading,
                                progress,
                                speed: None,
                                eta: None,
                                total_size: Some(format!("{:.2}MB", total as f64 / 1024.0 / 1024.0)),
                                downloaded_size: Some(format!(
                                    "{:.2}MB",
                                    downloaded as f64 / 1024.0 / 1024.0
                                )),
                                output_path: None,
                                error_message: None,
                            });
                        }
                    } else {
                        break;
                    }
                }

                file.flush().await.map_err(|e| format!("Flush error: {}", e))?;

                progress_callback(DownloadProgress {
                    task_id: url.clone(),
                    status: crate::database::models::DownloadStatus::Completed,
                    progress: 100.0,
                    speed: None,
                    eta: None,
                    total_size: None,
                    downloaded_size: None,
                    output_path: Some(output_path.to_string_lossy().to_string()),
                    error_message: None,
                });

                Ok::<(), String>(())
            }
            .await;

            if let Err(e) = result {
                progress_callback(DownloadProgress {
                    task_id: url.clone(),
                    status: crate::database::models::DownloadStatus::Failed,
                    progress: 0.0,
                    speed: None,
                    eta: None,
                    total_size: None,
                    downloaded_size: None,
                    output_path: None,
                    error_message: Some(e),
                });
            }
        });

        Ok(pid)
    }

    async fn cancel_download(&self, _pid: u32) -> Result<(), String> {
        // TODO: Implement cancellation logic with Arc<AtomicBool>
        Err("Cancellation not yet implemented for Douyin downloader".to_string())
    }

    async fn pause_download(&self, _pid: u32) -> Result<(), String> {
        Err("Pause not supported for Douyin downloader".to_string())
    }

    async fn resume_download(&self, _pid: u32) -> Result<(), String> {
        Err("Resume not supported for Douyin downloader".to_string())
    }
}
