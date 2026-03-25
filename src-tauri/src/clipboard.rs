use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::RwLock;

pub struct ClipboardWatcher {
    last_content: Arc<RwLock<String>>,
    enabled: Arc<RwLock<bool>>,
}

impl ClipboardWatcher {
    pub fn new() -> Self {
        Self {
            last_content: Arc::new(RwLock::new(String::new())),
            enabled: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
        log::info!("Clipboard watcher enabled: {}", enabled);
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
}

pub fn start_clipboard_watcher(
    watcher: Arc<ClipboardWatcher>,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        log::info!("Clipboard watcher started");
        loop {
            tokio::time::sleep(Duration::from_millis(1500)).await;

            if !watcher.is_enabled().await {
                continue;
            }

            let clipboard_text = match get_clipboard_text() {
                Some(text) => text.trim().to_string(),
                None => continue,
            };

            if clipboard_text.is_empty() {
                continue;
            }

            let last = watcher.last_content.read().await.clone();
            if clipboard_text == last {
                continue;
            }

            *watcher.last_content.write().await = clipboard_text.clone();

            if is_video_url(&clipboard_text) {
                log::info!("Clipboard detected video URL: {}", clipboard_text);
                let _ = app_handle.emit("clipboard-url", &clipboard_text);
            }
        }
    });
}

fn get_clipboard_text() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(["-Command", "Get-Clipboard"])
            .creation_flags(0x08000000)
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("pbpaste").output().ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .or_else(|_| {
                Command::new("xsel")
                    .args(["--clipboard", "--output"])
                    .output()
            })
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }
}

fn is_video_url(text: &str) -> bool {
    let text_lower = text.to_lowercase();

    // 必须是 URL 格式
    if !text_lower.starts_with("http://") && !text_lower.starts_with("https://") {
        return false;
    }

    // 不能有空格或换行
    if text.contains(' ') || text.contains('\n') {
        return false;
    }

    let video_domains = [
        "youtube.com", "youtu.be", "youtube-nocookie.com",
        "bilibili.com", "b23.tv",
        "twitter.com", "x.com",
        "tiktok.com",
        "instagram.com",
        "facebook.com", "fb.watch",
        "twitch.tv",
        "vimeo.com",
        "dailymotion.com",
        "nicovideo.jp",
        "reddit.com",
        "streamable.com",
        "v.qq.com",
        "ixigua.com",
        "douyin.com",
        "weibo.com",
        "zhihu.com",
    ];

    for domain in &video_domains {
        if text_lower.contains(domain) {
            return true;
        }
    }

    // 通用视频路径检测
    let video_patterns = ["/video/", "/watch", "/clip/", "/shorts/", "/reel/"];
    for pattern in &video_patterns {
        if text_lower.contains(pattern) {
            return true;
        }
    }

    false
}

#[cfg(target_os = "windows")]
trait CommandCreationFlags {
    fn creation_flags(&mut self, flags: u32) -> &mut Self;
}

#[cfg(target_os = "windows")]
impl CommandCreationFlags for std::process::Command {
    fn creation_flags(&mut self, flags: u32) -> &mut Self {
        use std::os::windows::process::CommandExt;
        std::os::windows::process::CommandExt::creation_flags(self, flags);
        self
    }
}