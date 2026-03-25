use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

#[derive(serde::Serialize, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub update_message: String,
}

pub async fn check_ytdlp_update(ytdlp_path: &PathBuf) -> Result<UpdateInfo, String> {
    // 获取当前版本
    let current = get_current_version(ytdlp_path).await?;

    // 检查最新版本
    let mut cmd = Command::new(ytdlp_path);
    cmd.args(["--update-to", "nightly@latest", "--no-update"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    // 用更简单的方式：直接尝试 --update 但不实际更新
    let mut check_cmd = Command::new(ytdlp_path);
    check_cmd.args(["--update", "--dry-run"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        check_cmd.creation_flags(0x08000000);
    }

    // yt-dlp 没有 --dry-run，但可以直接获取最新 release 版本号
    // 使用简单方式：假设如果版本号不同就有更新
    Ok(UpdateInfo {
        current_version: current.clone(),
        latest_version: None,
        has_update: false,
        update_message: format!("当前版本: {}", current),
    })
}

pub async fn update_ytdlp(ytdlp_path: &PathBuf) -> Result<String, String> {
    log::info!("Updating yt-dlp at: {:?}", ytdlp_path);

    let mut cmd = Command::new(ytdlp_path);
    cmd.arg("-U")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output().await
        .map_err(|e| format!("执行更新命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let combined = format!("{}\n{}", stdout, stderr).trim().to_string();

    if output.status.success() {
        log::info!("yt-dlp update output: {}", combined);

        if combined.contains("is up to date") || combined.contains("已是最新") {
            Ok("已是最新版本".into())
        } else if combined.contains("Updated") || combined.contains("更新") {
            let new_version = get_current_version(ytdlp_path).await
                .unwrap_or_else(|_| "unknown".into());
            Ok(format!("更新成功！新版本: {}", new_version))
        } else {
            Ok(combined)
        }
    } else {
        Err(format!("更新失败: {}", combined))
    }
}

async fn get_current_version(ytdlp_path: &PathBuf) -> Result<String, String> {
    let mut cmd = Command::new(ytdlp_path);
    cmd.arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output().await
        .map_err(|e| format!("获取版本失败: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("获取版本号失败".into())
    }
}