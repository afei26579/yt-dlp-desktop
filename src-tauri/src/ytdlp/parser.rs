use crate::database::models::{DownloadProgress, DownloadStatus};

pub fn parse_progress_line(task_id: &str, line: &str) -> Option<DownloadProgress> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    log::info!("[PARSER] {:?}", line);

    if line.starts_with("[download]") {
        let content = line.trim_start_matches("[download]").trim();

        if content.contains("has already been downloaded") {
            return Some(make_progress(task_id, DownloadStatus::Completed, 100.0, None, None, None, None, None));
        }

        if content.starts_with("Destination:") {
            return Some(make_progress(task_id, DownloadStatus::Downloading, 0.0, Some("开始下载..."), None, None, None, None));
        }

        if let Some(percent) = extract_percent(content) {
            let speed = extract_field(content, "at ", " ETA")
                .or_else(|| extract_field(content, "at ", " "));
            let eta = extract_after(content, "ETA ");
            let size = extract_field(content, "of ", " at")
                .or_else(|| extract_field(content, "of ", " in"))
                .map(|s| s.replace('~', "").trim().to_string());

            // 计算已下载大小
            let downloaded = size.as_ref()
                .and_then(|s| calculate_downloaded_size(s, percent));

            if percent >= 99.9 {
                return Some(DownloadProgress {
                    task_id: task_id.to_string(),
                    status: DownloadStatus::Downloading,
                    progress: 100.0,
                    speed: None,
                    eta: Some("00:00".into()),
                    total_size: size.clone(),
                    downloaded_size: size, // 100% 时已下载 = 总大小
                    output_path: None,
                    error_message: None,
                });
            }

            return Some(DownloadProgress {
                task_id: task_id.to_string(),
                status: DownloadStatus::Downloading,
                progress: percent,
                speed: speed.map(|s| s.trim().to_string()),
                eta: eta.map(|s| s.trim().to_string()),
                total_size: size,
                downloaded_size: downloaded,
                output_path: None,
                error_message: None,
            });
        }
    }

    if line.starts_with("[Merger]")
        || line.starts_with("[ffmpeg]")
        || line.starts_with("[ExtractAudio]")
        || line.starts_with("[VideoConvertor]")
    {
        return Some(make_progress(task_id, DownloadStatus::Merging, 99.0, None, Some("合并中..."), None, None, None));
    }

    if line.starts_with("ERROR:") || line.starts_with("error:") {
        return Some(DownloadProgress {
            task_id: task_id.to_string(),
            status: DownloadStatus::Failed,
            progress: 0.0,
            speed: None, eta: None, total_size: None, downloaded_size: None,
            output_path: None,
            error_message: Some(line.to_string()),
        });
    }

    None
}

/// 根据总大小和百分比计算已下载大小
fn calculate_downloaded_size(total_str: &str, percent: f64) -> Option<String> {
    let s = total_str.replace('~', "");
    let s = s.trim();
    if s.is_empty() || percent <= 0.0 {
        return None;
    }

    // 分离数字和单位: "96.72MiB" -> (96.72, "MiB")
    let num_end = s.find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());

    if num_end == 0 {
        return None;
    }

    let value: f64 = s[..num_end].parse().ok()?;
    let unit = s[num_end..].trim();
    let downloaded = value * percent / 100.0;

    Some(format!("{:.2}{}", downloaded, if unit.is_empty() { "" } else { unit }))
}

fn extract_percent(s: &str) -> Option<f64> {
    if let Some(pos) = s.find('%') {
        let before = &s[..pos];
        let num_str: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        if !num_str.is_empty() {
            return num_str.parse::<f64>().ok();
        }
    }
    None
}

fn extract_field(s: &str, start: &str, end: &str) -> Option<String> {
    let start_lower = s.to_lowercase();
    let start_marker = start.to_lowercase();
    let end_marker = end.to_lowercase();
    if let Some(i) = start_lower.find(&start_marker) {
        let after = &s[i + start.len()..];
        let after_lower = after.to_lowercase();
        if let Some(j) = after_lower.find(&end_marker) {
            let result = after[..j].trim().to_string();
            if !result.is_empty() { return Some(result); }
        }
    }
    None
}

fn extract_after(s: &str, marker: &str) -> Option<String> {
    let lower = s.to_lowercase();
    let marker_lower = marker.to_lowercase();
    if let Some(i) = lower.find(&marker_lower) {
        let after = s[i + marker.len()..].trim();
        if !after.is_empty() {
            let val = after.split_whitespace().next().unwrap_or(after);
            return Some(val.to_string());
        }
    }
    None
}

fn make_progress(
    task_id: &str, status: DownloadStatus, progress: f64,
    speed: Option<&str>, eta: Option<&str>, total_size: Option<&str>,
    downloaded_size: Option<&str>, error: Option<&str>,
) -> DownloadProgress {
    DownloadProgress {
        task_id: task_id.to_string(), status, progress,
        speed: speed.map(|s| s.to_string()),
        eta: eta.map(|s| s.to_string()),
        total_size: total_size.map(|s| s.to_string()),
        downloaded_size: downloaded_size.map(|s| s.to_string()),
        output_path: None,
        error_message: error.map(|s| s.to_string()),
    }
}