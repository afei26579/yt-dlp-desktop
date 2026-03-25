use std::path::PathBuf;
use std::process::Command;

/// 获取 yt-dlp 可执行文件路径
/// 按优先级依次查找：
/// 1. app_data_dir/bin/yt-dlp.exe  (用户安装目录)
/// 2. 程序同级目录/yt-dlp.exe     (便携模式)
/// 3. src-tauri/binaries/yt-dlp.exe (开发模式)
/// 4. 系统 PATH
pub fn get_ytdlp_path(app_dir: &PathBuf) -> Result<PathBuf, String> {
    let exe_name = if cfg!(target_os = "windows") { "yt-dlp.exe" } else { "yt-dlp" };
    let candidates = get_candidate_paths(app_dir, exe_name);
    log::info!("Searching for yt-dlp...");
    for path in &candidates {
        log::info!("  checking: {:?} exists={}", path, path.exists());
        if path.exists() {
            log::info!("✅ Found yt-dlp at: {:?}", path);
            return Ok(path.clone());
        }
    }
    if let Some(path) = find_in_system_path("yt-dlp") {
        log::info!("✅ Found yt-dlp in PATH: {:?}", path);
        return Ok(path);
    }
    Err(format!("yt-dlp not found. Searched:\n{}", candidates.iter().map(|p| format!("  - {}", p.display())).collect::<Vec<_>>().join("\n")))
}

/// 获取 ffmpeg 路径
pub fn get_ffmpeg_path(app_dir: &PathBuf) -> Option<PathBuf> {
    let exe_name = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };
    let candidates = get_candidate_paths(app_dir, exe_name);
    log::info!("Searching for ffmpeg...");
    for path in &candidates {
        log::info!("  checking: {:?} exists={}", path, path.exists());
        if path.exists() {
            log::info!("✅ Found ffmpeg at: {:?}", path);
            return Some(path.clone());
        }
    }
    if let Some(path) = find_in_system_path("ffmpeg") {
        log::info!("✅ Found ffmpeg in PATH: {:?}", path);
        return Some(path);
    }
    log::warn!("❌ ffmpeg not found in any location");
    None
}

/// 获取所有候选路径
fn get_candidate_paths(app_dir: &PathBuf, exe_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. app_data_dir/bin/
    paths.push(app_dir.join("bin").join(exe_name));

    // 2. 当前可执行文件同级目录
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            // 同级目录
            paths.push(exe_dir.join(exe_name));
            // 同级 bin/ 子目录
            paths.push(exe_dir.join("bin").join(exe_name));
            // 同级 binaries/ 子目录
            paths.push(exe_dir.join("binaries").join(exe_name));
        }
    }

    // 3. 开发模式：项目 src-tauri/binaries/
    //    当前工作目录可能是项目根目录或 src-tauri
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("binaries").join(exe_name));
        paths.push(cwd.join("src-tauri").join("binaries").join(exe_name));
    }

    // 4. 开发模式：通过 CARGO_MANIFEST_DIR 环境变量
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        paths.push(PathBuf::from(&manifest_dir).join("binaries").join(exe_name));
    }

    paths
}

/// 在系统 PATH 中查找可执行文件
fn find_in_system_path(name: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("where")
            .arg(name)
            .output()
            .ok()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()?
                .trim()
                .to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("which")
            .arg(name)
            .output()
            .ok()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    None
}

/// 获取 yt-dlp 版本
pub fn get_ytdlp_version(ytdlp_path: &PathBuf) -> Result<String, String> {
    let output = Command::new(ytdlp_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Failed to get yt-dlp version".to_string())
    }
}