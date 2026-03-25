use crate::database::models::DownloadTask;
use std::path::Path;

pub fn export_history_json(tasks: &[DownloadTask], path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    log::info!("Exported {} tasks to JSON: {}", tasks.len(), path);
    Ok(())
}

pub fn export_history_csv(tasks: &[DownloadTask], path: &str) -> Result<(), String> {
    let mut lines = Vec::new();
    lines.push("ID,标题,URL,状态,画质,格式,文件路径,创建时间,完成时间,错误信息".to_string());

    for task in tasks {
        let status = serde_json::to_string(&task.status).unwrap_or_default();
        let line = format!(
            "\"{}\",\"{}\",\"{}\",{},\"{}\",{},\"{}\",\"{}\",\"{}\",\"{}\"",
            escape_csv(&task.id),
            escape_csv(&task.title),
            escape_csv(&task.url),
            status.replace('"', ""),
            escape_csv(&task.quality_label),
            if task.audio_only { "音频" } else { "视频" },
            escape_csv(&task.output_path.clone().unwrap_or_default()),
            escape_csv(&task.created_at),
            escape_csv(&task.completed_at.clone().unwrap_or_default()),
            escape_csv(&task.error_message.clone().unwrap_or_default()),
        );
        lines.push(line);
    }

    // 写入 UTF-8 BOM + CSV 内容（兼容 Excel）
    let bom = "\u{FEFF}";
    let content = format!("{}{}", bom, lines.join("\n"));
    std::fs::write(path, content)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    log::info!("Exported {} tasks to CSV: {}", tasks.len(), path);
    Ok(())
}

pub fn import_urls_from_file(path: &str) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let urls: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with("//")
                && (line.starts_with("http://") || line.starts_with("https://"))
        })
        .collect();

    if urls.is_empty() {
        return Err("文件中没有找到有效的 URL".into());
    }

    log::info!("Imported {} URLs from file: {}", urls.len(), path);
    Ok(urls)
}

pub fn export_settings(settings: &crate::database::models::AppSettings, path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

pub fn import_settings(path: &str) -> Result<crate::database::models::AppSettings, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读取失败: {}", e))?;
    let settings: crate::database::models::AppSettings = serde_json::from_str(&content)
        .map_err(|e| format!("解析失败: {}", e))?;
    Ok(settings)
}

fn escape_csv(s: &str) -> String {
    s.replace('"', "\"\"")
}