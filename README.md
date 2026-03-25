# YT-DLP Desktop

<div align="center">

一款轻量级跨平台视频下载工具，为 yt-dlp 提供直观的图形界面

[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](https://github.com/yourusername/yt-dlp-desktop)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)]()

[English](#english) | [中文](#chinese)

</div>

---

## Chinese

### 简介

YT-DLP Desktop 是一款面向普通用户的轻量级视频下载工具，基于强大的 [yt-dlp](https://github.com/yt-dlp/yt-dlp) 命令行工具开发。通过直观的图形界面，降低使用门槛，让非技术用户也能一键下载来自 1000+ 网站的在线视频。

### 功能特性

- **简单易用**：粘贴链接 → 一键下载，无需命令行操作
- **格式选择**：支持多种视频/音频格式和画质选择
- **批量下载**：支持多链接和播放列表批量下载
- **下载队列**：智能队列管理，可配置并发下载数
- **字幕提取**：自动下载视频字幕，支持多语言
- **音频提取**：一键提取音频为 MP3/FLAC 格式
- **剪贴板监控**：自动检测复制的视频链接
- **下载历史**：本地数据库记录所有下载任务
- **跨平台**：支持 Windows、macOS 和 Linux
- **轻量高效**：基于 Tauri 2 构建，体积小、内存占用低

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 + TypeScript + Vite + Pinia |
| 后端 | Rust + Tauri 2 |
| 数据库 | SQLite (rusqlite) |
| 核心引擎 | yt-dlp + ffmpeg |

### 快速开始

#### 前置要求

- Node.js 18+ 和 npm
- Rust 工具链 (Rust 1.70+)
- 系统构建工具
  - Windows: Visual Studio Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: build-essential, libwebkit2gtk-4.1-dev

#### 安装依赖

```bash
npm install
```

#### 开发模式

```bash
npm run tauri dev
```

#### 构建发布版本

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

### 项目结构

```
yt-dlp-desktop/
├── src/                    # Vue 3 前端
│   ├── views/             # 页面组件
│   ├── components/        # 可复用组件
│   ├── stores/            # Pinia 状态管理
│   └── utils/             # 工具函数
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── commands/      # Tauri IPC 命令
│   │   ├── ytdlp/         # yt-dlp 集成
│   │   ├── queue.rs       # 下载队列管理
│   │   ├── database/      # SQLite 数据库
│   │   └── config.rs      # 配置管理
│   └── binaries/          # 外部工具 (需手动添加)
└── package.json
```

### 外部工具设置

应用需要以下二进制文件，需手动放置到 `src-tauri/binaries/` 目录：

| 文件 | 来源 | 大小 |
|------|------|------|
| yt-dlp.exe / yt-dlp | [yt-dlp releases](https://github.com/yt-dlp/yt-dlp/releases) | ~15MB |
| ffmpeg.exe / ffmpeg | [FFmpeg builds](https://www.gyan.dev/ffmpeg/builds/) | ~80MB |

或让应用自动下载到系统目录。

### 清理应用数据

```powershell
# Windows
Remove-Item "$env:APPDATA\com.ytdlp.desktop" -Recurse -ErrorAction SilentlyContinue

# macOS
rm -rf ~/Library/Application\ Support/com.ytdlp.desktop

# Linux
rm -rf ~/.config/com.ytdlp.desktop
```

### 开发指南

#### 类型安全

- 所有 Tauri 命令使用 TypeScript 接口定义（`src/utils/invoke.ts`）
- Rust 结构体使用 `#[derive(Serialize, Deserialize)]` 进行序列化

#### 异步运行时

- 使用 `tauri::async_runtime::spawn()` 而非 `tokio::spawn()`
- 共享状态使用 `Arc<T>` 包装（Database、ConfigManager、DownloadQueue）

#### 数据库迁移

```rust
// 自动添加播放列表字段
db.execute("ALTER TABLE download_tasks ADD COLUMN playlist_title TEXT", [])?;
```

### 支持的网站

支持 1000+ 网站，包括：

- YouTube（含 Shorts、播放列表）
- Bilibili
- 抖音 / TikTok
- 哔哩哔哩
- Vimeo
- Twitter/X
- Instagram
- ...

完整列表请查看 [supported_sites.md](supportedsites.md)

### 路线图

- [ ] 代理设置
- [ ] Cookie 导入
- [ ] 自定义文件命名模板
- [ ] 高级命令行参数配置
- [ ] 应用自动更新
- [ ] 多语言支持扩展

### 贡献

欢迎提交 Issue 和 Pull Request！

### 许可证

详见 [LICENSE](LICENSE)

---

## English

### Introduction

YT-DLP Desktop is a lightweight cross-platform video downloader with a graphical interface for [yt-dlp](https://github.com/yt-dlp/yt-dlp). It lowers the barrier to entry, allowing non-technical users to download online videos from 1000+ websites with just one click.

### Features

- **Simple to use**: Paste URL → Download, no command line needed
- **Format selection**: Multiple video/audio formats and quality options
- **Batch downloads**: Multi-link and playlist support
- **Download queue**: Smart queue management with configurable concurrency
- **Subtitles**: Automatic subtitle download with multi-language support
- **Audio extraction**: One-click extraction to MP3/FLAC
- **Clipboard monitoring**: Auto-detect copied video URLs
- **Download history**: Local database tracks all tasks
- **Cross-platform**: Windows, macOS, Linux
- **Lightweight**: Built with Tauri 2, small size, low memory footprint

### Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Vue 3 + TypeScript + Vite + Pinia |
| Backend | Rust + Tauri 2 |
| Database | SQLite (rusqlite) |
| Core Engine | yt-dlp + ffmpeg |

### Quick Start

#### Prerequisites

- Node.js 18+ and npm
- Rust toolchain (Rust 1.70+)
- System build tools

#### Install Dependencies

```bash
npm install
```

#### Development Mode

```bash
npm run tauri dev
```

#### Build Release

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/`

### External Tools Setup

Place these binaries in `src-tauri/binaries/`:

| File | Source | Size |
|------|--------|------|
| yt-dlp.exe / yt-dlp | [yt-dlp releases](https://github.com/yt-dlp/yt-dlp/releases) | ~15MB |
| ffmpeg.exe / ffmpeg | [FFmpeg builds](https://www.gyan.dev/ffmpeg/builds/) | ~80MB |

Or let the app auto-download to system directories.

### Clear App Data

```powershell
# Windows
Remove-Item "$env:APPDATA\com.ytdlp.desktop" -Recurse -ErrorAction SilentlyContinue

# macOS
rm -rf ~/Library/Application\ Support/com.ytdlp.desktop

# Linux
rm -rf ~/.config/com.ytdlp.desktop
```

### Contributing

Issues and Pull Requests are welcome!

### License

see [LICENSE](LICENSE)
