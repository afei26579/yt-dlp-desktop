use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::database::models::*;
use super::parser;

pub struct ProcessManager {
    active_processes: Arc<RwLock<HashMap<String, u32>>>,
    paused_set: Arc<RwLock<HashSet<String>>>,
    cancelled_set: Arc<RwLock<HashSet<String>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            active_processes: Arc::new(RwLock::new(HashMap::new())),
            paused_set: Arc::new(RwLock::new(HashSet::new())),
            cancelled_set: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    // ===== 视频信息 =====

        pub async fn fetch_video_info(&self, ytdlp_path: &PathBuf, url: &str, settings: &AppSettings) -> Result<VideoInfo, String> {
        // ===== 第一次尝试：使用用户当前配置 =====
        match self.fetch_video_info_inner(ytdlp_path, url, settings, true).await {
            Ok(info) => return Ok(info),
            Err(first_error) => {
                log::warn!("First attempt failed: {}", first_error);

                let is_douyin = url.contains("douyin.com") || url.contains("tiktok.com");
                let needs_fresh_cookie = first_error.contains("Fresh cookies")
                    || first_error.contains("fresh cookies");
                let needs_cookie = needs_fresh_cookie
                    || first_error.contains("cookie") || first_error.contains("Cookie")
                    || first_error.contains("Sign in") || first_error.contains("not a bot");

                // ===== 抖音专属：Fresh cookies 重试 =====
                if is_douyin && needs_fresh_cookie {
                    log::info!("Douyin fresh cookies error, trying specialized strategies...");

                    // 策略1：加 User-Agent + Referer 模拟真实浏览器
                    let browsers_to_try = ["chrome", "edge", "firefox", "brave"];
                    for browser in &browsers_to_try {
                        log::info!("Douyin retry: --cookies-from-browser {} + headers", browser);
                        let mut retry_settings = settings.clone();
                        retry_settings.use_browser_cookie = true;
                        retry_settings.browser_type = browser.to_string();
                        retry_settings.cookie_file_path = None;

                        match self.fetch_video_info_douyin(ytdlp_path, url, &retry_settings).await {
                            Ok(info) => {
                                log::info!("✅ Douyin succeeded with browser: {}", browser);
                                return Ok(info);
                            }
                            Err(e) => {
                                log::warn!("Douyin retry with {} failed: {}", browser, e);
                            }
                        }
                    }

                    // 策略2：如果有 cookie 文件，也试一下
                    if let Some(ref cookie_path) = settings.cookie_file_path {
                        if !cookie_path.is_empty() && std::path::Path::new(cookie_path).exists() {
                            log::info!("Douyin retry: cookie file + headers");
                            match self.fetch_video_info_douyin(ytdlp_path, url, settings).await {
                                Ok(info) => return Ok(info),
                                Err(e) => log::warn!("Douyin retry with cookie file failed: {}", e),
                            }
                        }
                    }

                    // 全部失败，返回友好错误
                    return Err(format!(
                        "DOUYIN_FRESH_COOKIE|{}",
                        first_error
                    ));
                }

                // ===== 通用 cookie 重试 =====
                if needs_cookie && !settings.use_browser_cookie && settings.cookie_file_path.is_none() {
                    let browsers = ["edge", "chrome", "firefox", "brave"];
                    for browser in &browsers {
                        let mut rs = settings.clone();
                        rs.use_browser_cookie = true;
                        rs.browser_type = browser.to_string();
                        if let Ok(info) = self.fetch_video_info_inner(ytdlp_path, url, &rs, true).await {
                            return Ok(info);
                        }
                    }
                    return Err(format!("{}\n\n💡 该网站需要登录验证。", first_error));
                }

                Err(first_error)
            }
        }
    }

    /// 抖音专属：带额外 headers 的解析
    async fn fetch_video_info_douyin(
        &self, ytdlp_path: &PathBuf, url: &str, settings: &AppSettings,
    ) -> Result<VideoInfo, String> {
        let mut cmd = Command::new(ytdlp_path);
        cmd.env("PYTHONIOENCODING", "utf-8").env("PYTHONUTF8", "1");
        cmd.args(["--dump-json", "--no-download"])
            .arg("--extractor-retries").arg("3")
            .arg("--user-agent").arg(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
            )
            .arg("--add-header").arg("Referer: https://www.douyin.com/")
            .arg("--add-header").arg("Accept-Language: zh-CN,zh;q=0.9")
            .arg(url)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }

        Self::apply_proxy_settings(&mut cmd, settings);
        Self::apply_cookie_settings(&mut cmd, settings);

        let output = cmd.output().await.map_err(|e| format!("Failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("yt-dlp error: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("Parse error: {}", e))?;
        let formats = Self::parse_formats(&json);

        Ok(VideoInfo {
            id: json["id"].as_str().unwrap_or("").into(),
            title: json["title"].as_str().unwrap_or("Unknown").into(),
            url: url.into(),
            thumbnail: json["thumbnail"].as_str().map(|s| s.into()),
            duration: json["duration"].as_f64(),
            uploader: json["uploader"].as_str().map(|s| s.into()),
            upload_date: json["upload_date"].as_str().map(|s| s.into()),
            description: json["description"].as_str().map(|s| s.into()),
            webpage_url: json["webpage_url"].as_str().unwrap_or(url).into(),
            formats,
            is_playlist: json["_type"].as_str() == Some("playlist"),
            playlist_count: json["playlist_count"].as_u64().map(|n| n as u32),
            entries: Vec::new(),
        })
    }

    async fn fetch_video_info_inner(
        &self, ytdlp_path: &PathBuf, url: &str, settings: &AppSettings, apply_cookies: bool,
    ) -> Result<VideoInfo, String> {
        let mut cmd = Command::new(ytdlp_path);
        cmd.env("PYTHONIOENCODING", "utf-8").env("PYTHONUTF8", "1");
        cmd.args(["--dump-json", "--no-download", "--flat-playlist"]);
        cmd.arg(url).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }
        Self::apply_proxy_settings(&mut cmd, settings);
        if apply_cookies { Self::apply_cookie_settings(&mut cmd, settings); }
        let output = cmd.output().await.map_err(|e| format!("Failed: {}", e))?;
        if !output.status.success() {
            return Err(format!("yt-dlp error: {}", String::from_utf8_lossy(&output.stderr).trim()));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.len() > 1 { return self.parse_playlist_output(url, &lines); }
        let json: serde_json::Value = serde_json::from_str(lines.first().unwrap_or(&"{}"))
            .map_err(|e| format!("Parse error: {}", e))?;
        if json["_type"].as_str() == Some("playlist") { return self.parse_playlist_json(url, &json); }
        let formats = Self::parse_formats(&json);
        Ok(VideoInfo {
            id: json["id"].as_str().unwrap_or("").into(),
            title: json["title"].as_str().unwrap_or("Unknown").into(),
            url: url.into(), thumbnail: json["thumbnail"].as_str().map(|s| s.into()),
            duration: json["duration"].as_f64(), uploader: json["uploader"].as_str().map(|s| s.into()),
            upload_date: json["upload_date"].as_str().map(|s| s.into()),
            description: json["description"].as_str().map(|s| s.into()),
            webpage_url: json["webpage_url"].as_str().unwrap_or(url).into(),
            formats, is_playlist: false, playlist_count: None, entries: Vec::new(),
        })
    }

    fn parse_playlist_output(&self, url: &str, lines: &[&str]) -> Result<VideoInfo, String> {
        let mut entries = Vec::new();
        let mut pl_title = String::from("播放列表");
        let mut pl_uploader: Option<String> = None;
        for (i, line) in lines.iter().enumerate() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if i == 0 {
                    pl_title = json["playlist_title"].as_str().or(json["playlist"].as_str())
                        .unwrap_or("播放列表").to_string();
                    pl_uploader = json["playlist_uploader"].as_str().or(json["uploader"].as_str()).map(|s| s.into());
                }
                entries.push(PlaylistEntry {
                    id: json["id"].as_str().unwrap_or("").into(),
                    title: json["title"].as_str().or(json["fulltitle"].as_str()).unwrap_or("Unknown").into(),
                    url: json["webpage_url"].as_str().or(json["url"].as_str()).unwrap_or("").into(),
                    thumbnail: json["thumbnail"].as_str().map(|s| s.into()),
                    duration: json["duration"].as_f64(),
                    uploader: json["uploader"].as_str().map(|s| s.into()),
                    index: (i + 1) as u32,
                });
            }
        }
        let count = entries.len() as u32;
        Ok(VideoInfo {
            id: "playlist".into(), title: pl_title, url: url.into(),
            thumbnail: entries.first().and_then(|e| e.thumbnail.clone()),
            duration: None, uploader: pl_uploader, upload_date: None, description: None,
            webpage_url: url.into(),
            formats: vec![FormatInfo { format_id: "best".into(), format_note: Some("Best".into()),
                ext: "mp4".into(), resolution: None, filesize: None, filesize_approx: None,
                vcodec: None, acodec: None, quality_label: "最佳画质".into() }],
            is_playlist: true, playlist_count: Some(count), entries,
        })
    }

    fn parse_playlist_json(&self, url: &str, json: &serde_json::Value) -> Result<VideoInfo, String> {
        let mut entries = Vec::new();
        if let Some(list) = json["entries"].as_array() {
            for (i, e) in list.iter().enumerate() {
                entries.push(PlaylistEntry {
                    id: e["id"].as_str().unwrap_or("").into(),
                    title: e["title"].as_str().unwrap_or("Unknown").into(),
                    url: e["webpage_url"].as_str().or(e["url"].as_str()).unwrap_or("").into(),
                    thumbnail: e["thumbnail"].as_str().map(|s| s.into()),
                    duration: e["duration"].as_f64(),
                    uploader: e["uploader"].as_str().map(|s| s.into()),
                    index: (i + 1) as u32,
                });
            }
        }
        let count = entries.len() as u32;
        Ok(VideoInfo {
            id: json["id"].as_str().unwrap_or("playlist").into(),
            title: json["title"].as_str().unwrap_or("播放列表").into(),
            url: url.into(),
            thumbnail: json["thumbnail"].as_str().map(|s| s.into())
                .or_else(|| entries.first().and_then(|e| e.thumbnail.clone())),
            duration: None, uploader: json["uploader"].as_str().map(|s| s.into()),
            upload_date: None, description: json["description"].as_str().map(|s| s.into()),
            webpage_url: json["webpage_url"].as_str().unwrap_or(url).into(),
            formats: vec![FormatInfo { format_id: "best".into(), format_note: Some("Best".into()),
                ext: "mp4".into(), resolution: None, filesize: None, filesize_approx: None,
                vcodec: None, acodec: None, quality_label: "最佳画质".into() }],
            is_playlist: true, playlist_count: Some(count), entries,
        })
    }

    /// 诊断 cookie 读取失败的具体原因
    fn diagnose_cookie_failure(err_msg: &str, browser_type: &str) {
        log::error!("=== Cookie Diagnosis ===");
        log::error!("  Browser: {}", browser_type);

        let err_lower = err_msg.to_lowercase();

        if err_lower.contains("could not find a suitable cookiejar") || err_lower.contains("no cookies were found") {
            log::error!("  ❌ 诊断: yt-dlp 找不到浏览器 cookie 数据库文件");
            log::error!("  💡 可能原因:");
            log::error!("     1. 浏览器从未访问过该网站");
            log::error!("     2. 浏览器使用了非默认 Profile");
            log::error!("     3. 浏览器安装位置非标准");
        }

        if err_lower.contains("permission") || err_lower.contains("access") || err_lower.contains("locked") {
            log::error!("  ❌ 诊断: Cookie 数据库被锁定或无权限访问");
            log::error!("  💡 可能原因:");
            log::error!("     1. 浏览器正在运行（Chrome 会锁定 cookie 数据库）");
            log::error!("     2. 杀毒软件阻止了访问");
            log::error!("  💡 解决方案: 关闭浏览器后再试，或使用 cookie 文件方式");
        }

        if err_lower.contains("decrypt") || err_lower.contains("crypt") || err_lower.contains("keyring") || err_lower.contains("secretstorage") {
            log::error!("  ❌ 诊断: Cookie 解密失败");
            log::error!("  💡 可能原因:");
            log::error!("     1. Chrome v127+ 使用了 App-Bound Encryption，yt-dlp 无法解密");
            log::error!("     2. 缺少系统密钥环支持");
            log::error!("  💡 解决方案: 使用 cookie 文件方式（推荐安装浏览器扩展导出 cookies.txt）");
        }

        if err_lower.contains("profile") {
            log::error!("  ❌ 诊断: 浏览器 Profile 相关问题");
            log::error!("  💡 可能原因: 使用了多个 Chrome Profile，yt-dlp 读取了错误的 Profile");
        }

        // 通用建议
        log::error!("  === 通用排查步骤 ===");
        log::error!("  1. 确认已在 {} 浏览器中登录目标网站", browser_type);
        log::error!("  2. 尝试关闭 {} 浏览器后重试", browser_type);
        log::error!("  3. 如持续失败，建议使用 Cookie 文件方式");
        log::error!("  4. 安装浏览器扩展 'Get cookies.txt LOCALLY' 导出 cookie 文件");
    }

    // ===== 下载 =====

    pub async fn start_download(
        &self,
        ytdlp_path: &PathBuf,
        ffmpeg_path: &Option<PathBuf>,
        task: &DownloadTask,
        settings: &AppSettings,
        is_resume: bool,
        progress_sender: tokio::sync::mpsc::UnboundedSender<DownloadProgress>,
    ) -> Result<bool, String> {
        let download_dir = &settings.download_path;
        if !download_dir.is_empty() { std::fs::create_dir_all(download_dir).ok(); }

        // 清除取消标记
        self.cancelled_set.write().await.remove(&task.id);

        let mut cmd = Command::new(ytdlp_path);
        cmd.env("PYTHONIOENCODING", "utf-8").env("PYTHONUTF8", "1");

        let output_template = format!("{}/{}", download_dir.replace('\\', "/"), settings.filename_template);

        cmd.arg(&task.url)
            .arg("-o").arg(&output_template)
            .arg("--newline")
            .arg("--no-colors")
            .arg("--progress")
            .arg("--windows-filenames")
            .arg("--encoding").arg("utf-8")
            .arg("--print").arg("after_move:filepath")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }

        // ★ 修复5: 续传用 -c，新下载用 --force-overwrites
        if is_resume {
            cmd.arg("-c");
        } else {
            cmd.arg("--force-overwrites");
        }

        if let Some(ref ffmpeg) = ffmpeg_path {
            cmd.arg("--ffmpeg-location").arg(ffmpeg.parent().unwrap_or(ffmpeg));
        }

        if task.audio_only {
            cmd.args(["-x", "--audio-format", "mp3"]);
            cmd.arg("--audio-quality").arg(&settings.audio_quality);
        } else if let Some(ref fid) = task.format_id {
            cmd.arg("-f").arg(format!("{}+bestaudio/best", fid));
        } else {
            cmd.arg("-f").arg("bestvideo+bestaudio/best");
        }
        cmd.args(["--merge-output-format", "mp4"]);

        if task.download_subtitle {
            cmd.arg("--write-sub").arg("--write-auto-sub");
            if let Some(ref l) = task.subtitle_lang { cmd.arg("--sub-lang").arg(l); }
            cmd.arg("--embed-subs");
        }
        if let Some(ref limit) = settings.speed_limit {
            if !limit.is_empty() { cmd.arg("--limit-rate").arg(limit); }
        }
        if settings.download_thumbnail { cmd.arg("--write-thumbnail"); }
        if settings.download_metadata { cmd.arg("--write-info-json"); }

        Self::apply_proxy_settings(&mut cmd, settings);
        Self::apply_cookie_settings(&mut cmd, settings);
        if let Some(ref extra) = settings.extra_args {
            let args: Vec<&str> = extra.split_whitespace().collect();
            if !args.is_empty() { cmd.args(args); }
        }

        log::info!("Starting download (resume={}): {}", is_resume, task.url);

        let mut child = cmd.spawn().map_err(|e| format!("Failed: {}", e))?;
        let task_id = task.id.clone();

        if let Some(pid) = child.id() {
            self.active_processes.write().await.insert(task_id.clone(), pid);
        }

        let stderr_lines: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        let output_filepath: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

        // ★ 修复1: 进度分段追踪
        let dest_count = Arc::new(AtomicU32::new(0));
        let cancelled_set = self.cancelled_set.clone();

        // stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let sender = progress_sender.clone();
            let tid = task_id.clone();
            let fp = output_filepath.clone();
            let dc = dest_count.clone();
            let cs = cancelled_set.clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    if cs.read().await.contains(&tid) { continue; }
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() { continue; }

                    // 计数 Destination
                    if trimmed.contains("[download] Destination:") {
                        dc.fetch_add(1, Ordering::SeqCst);
                    }

                    if let Some(mut p) = parser::parse_progress_line(&tid, &trimmed) {
                        adjust_progress(&mut p, dc.load(Ordering::SeqCst));
                        let _ = sender.send(p);
                        continue;
                    }

                    if !trimmed.starts_with('[') && !trimmed.starts_with("ERROR") && !trimmed.starts_with("WARNING") {
                        let is_path = trimmed.contains('/') || trimmed.contains('\\')
                            || trimmed.ends_with(".mp4") || trimmed.ends_with(".mp3")
                            || trimmed.ends_with(".mkv") || trimmed.ends_with(".webm")
                            || trimmed.ends_with(".m4a");
                        if is_path { *fp.write().await = Some(trimmed); }
                    }
                }
            });
        }

        // stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let sender = progress_sender.clone();
            let tid = task_id.clone();
            let sc = stderr_lines.clone();
            let dc = dest_count.clone();
            let cs = cancelled_set.clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    if cs.read().await.contains(&tid) { continue; }
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() { continue; }
                    sc.write().await.push(trimmed.clone());

                    if trimmed.contains("[download] Destination:") {
                        dc.fetch_add(1, Ordering::SeqCst);
                    }

                    if let Some(mut p) = parser::parse_progress_line(&tid, &trimmed) {
                        adjust_progress(&mut p, dc.load(Ordering::SeqCst));
                        let _ = sender.send(p);
                    }
                }
            });
        }

        let status = child.wait().await.map_err(|e| format!("Process error: {}", e))?;
        self.active_processes.write().await.remove(&task_id);

        // ★ 修复4: 被取消的直接返回
        if self.cancelled_set.read().await.contains(&task_id) {
            self.cancelled_set.write().await.remove(&task_id);
            return Err("Cancelled".into());
        }

        // ★ 修复3: 检查是否被暂停
        let was_paused = self.paused_set.write().await.remove(&task_id);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        if status.success() {
            let final_path = output_filepath.read().await.clone();
            let _ = progress_sender.send(DownloadProgress {
                task_id: task_id.clone(), status: DownloadStatus::Completed, progress: 100.0,
                speed: None, eta: None, total_size: None, downloaded_size: None,
                output_path: final_path, error_message: None,
            });
            Ok(true)
        } else if was_paused {
            log::info!("Task {} paused", task_id);
            let _ = progress_sender.send(DownloadProgress {
                task_id: task_id.clone(), status: DownloadStatus::Paused, progress: 0.0,
                speed: None, eta: Some("已暂停".into()), total_size: None,
                downloaded_size: None, output_path: None, error_message: None,
            });
            Ok(false)
        } else {
            let collected = stderr_lines.read().await;
            let err = collected.iter()
                .filter(|l| l.contains("ERROR") || l.contains("error"))
                .cloned().collect::<Vec<_>>().join("\n");
            let final_err = if err.is_empty() { "Download failed".into() } else { err };
            let _ = progress_sender.send(DownloadProgress {
                task_id: task_id.clone(), status: DownloadStatus::Failed, progress: 0.0,
                speed: None, eta: None, total_size: None, downloaded_size: None,
                output_path: None, error_message: Some(final_err.clone()),
            });
            Err(final_err)
        }
    }

    pub async fn pause_download(&self, task_id: &str) -> Result<(), String> {
        self.paused_set.write().await.insert(task_id.to_string());
        self.kill_process(task_id).await
    }

    pub async fn cancel_download(&self, task_id: &str) -> Result<(), String> {
        self.paused_set.write().await.remove(task_id);
        self.cancelled_set.write().await.insert(task_id.to_string());
        self.kill_process(task_id).await
    }

    async fn kill_process(&self, task_id: &str) -> Result<(), String> {
        if let Some(pid) = self.active_processes.write().await.remove(task_id) {
            log::info!("Killing PID {} for task {}", pid, task_id);
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F", "/T"])
                    .output(); // ★ 用 output() 等待完成
            }
            #[cfg(not(target_os = "windows"))]
            {
                unsafe { libc::kill(pid as i32, libc::SIGTERM); }
            }
        }
        Ok(())
    }

    fn apply_proxy_settings(cmd: &mut Command, s: &AppSettings) {
        match s.proxy_mode {
            ProxyMode::Custom => { if let Some(ref u) = s.proxy_url { if !u.is_empty() { cmd.arg("--proxy").arg(u); } } }
            ProxyMode::System => {}
            ProxyMode::None => { cmd.arg("--proxy").arg(""); }
        }
    }

    fn apply_cookie_settings(cmd: &mut Command, s: &AppSettings) {
        if let Some(ref p) = s.cookie_file_path {
            if !p.is_empty() && std::path::Path::new(p).exists() { cmd.arg("--cookies").arg(p); return; }
        }
        if s.use_browser_cookie { cmd.arg("--cookies-from-browser").arg(&s.browser_type); }
    }

    pub fn parse_formats(json: &serde_json::Value) -> Vec<FormatInfo> {
        let mut formats = Vec::new();
        let mut seen = std::collections::HashSet::new();
        if let Some(list) = json["formats"].as_array() {
            for f in list.iter().rev() {
                let vc = f["vcodec"].as_str().unwrap_or("none");
                if vc == "none" { continue; }
                let ql = if let Some(h) = f["height"].as_u64() {
                    let l = format!("{}p", h); if !seen.insert(l.clone()) { continue; } l
                } else { continue; };
                let vs = f["filesize"].as_u64().or(f["filesize_approx"].as_u64()).unwrap_or(0);
                let est = if vs > 0 { (vs as f64 * 1.2) as u64 } else { 0 };
                formats.push(FormatInfo {
                    format_id: f["format_id"].as_str().unwrap_or("").into(),
                    format_note: f["format_note"].as_str().map(|s| s.into()), ext: "mp4".into(),
                    resolution: f["resolution"].as_str().map(|s| s.into()),
                    filesize: if est > 0 { Some(est) } else { None },
                    filesize_approx: f["filesize_approx"].as_u64(),
                    vcodec: Some(vc.into()), acodec: f["acodec"].as_str().map(|s| s.into()),
                    quality_label: ql,
                });
            }
        }
        if formats.is_empty() {
            formats.push(FormatInfo { format_id: "best".into(), format_note: Some("Best".into()),
                ext: "mp4".into(), resolution: None, filesize: None, filesize_approx: None,
                vcodec: None, acodec: None, quality_label: "最佳画质".into() });
        }
        formats
    }
}

/// ★ 修复1: 根据 Destination 计数调整进度，防止双流下载时进度回零
fn adjust_progress(p: &mut DownloadProgress, dest_count: u32) {
    if p.status != DownloadStatus::Downloading { return; }

    match dest_count {
        0 | 1 => {
            // 单流或第一流：0% → 95%
            p.progress = p.progress * 0.95;
        }
        2 => {
            // 第二流（音频）: 0% → 95% 映射到 48% → 95%
            // 判断是否第一流还是第二流：如果 raw < 5% 且 dest_count==2 说明刚进入第二流
            // 简单处理：第一流映射 0→47，第二流映射 47→95
            // 但我们无法区分当前是第几流，用 Destination 计数：
            // dest_count 从1变2时进入第二流，之后的进度都是第二流
            p.progress = 47.0 + p.progress * 0.48;
        }
        _ => {
            // 3流以上（少见）
            p.progress = p.progress * 0.95;
        }
    }

    if p.progress > 95.0 { p.progress = 95.0; }
}