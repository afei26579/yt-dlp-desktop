# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

YT-DLP Desktop is a lightweight cross-platform desktop application that provides a GUI for yt-dlp video downloading. Built with Tauri 2, Vue 3, TypeScript, and Rust.

**Technology Stack:**
- Frontend: Vue 3 + TypeScript + Vite + Pinia
- Backend: Rust + Tauri 2
- Database: SQLite (via rusqlite)
- External Tools: yt-dlp (core), ffmpeg (format merging)

**Architecture:**
```
UI Layer (Vue 3) → Pinia Stores → Tauri IPC → Rust Commands → yt-dlp/ffmpeg
```

## Common Development Commands

```bash
# Start development mode (builds Rust + runs Vite dev server)
npm run tauri dev

# Build for release
npm run tauri build

# Type checking (no output)
npm run build

# Vite dev server only (for frontend-only work)
npm run dev
```

**Clear app data/config:**
```powershell
# Windows
Remove-Item "$env:APPDATA\com.ytdlp.desktop" -Recurse -ErrorAction SilentlyContinue
```

## Architecture & Code Organization

### Frontend Structure (`src/`)
- **Views**: `DownloadView.vue`, `QueueView.vue`, `SettingsView.vue` - main tab pages
- **Components**: Reusable Vue components (VideoPreview, ProgressBar, DownloadOptions, etc.)
- **Stores**: Pinia stores manage state
  - `download.ts`: Download queue, history, active tasks, progress tracking
  - `settings.ts`: App settings, loaded from/saved to Rust backend
- **Utils**:
  - `invoke.ts`: Type-safe Tauri command wrappers
  - `i18n.ts`: Internationalization (Chinese/English)
  - `errors.ts`: Error classification for user-friendly messages
  - `theme.ts`: Theme management (light/dark/system)

### Backend Structure (`src-tauri/src/`)
- **Commands** (`commands/`): Tauri IPC endpoints
  - `download.rs`: fetch_video_info, start_download, cancel_download, pause_download, resume_download, retry_download
  - `settings.rs`: getSettings, saveSettings, updateMaxConcurrent
  - `video_info.rs`: (placeholder, not currently registered)
- **Modules**:
  - `lib.rs`: Main Tauri app setup, state management, tray icon, startup logic
  - `queue.rs`: Download queue with concurrent task limiting (VecDeque-based)
  - `database/mod.rs`: SQLite operations (insert, update, fetch history)
  - `clipboard.rs`: Cross-platform clipboard monitoring for URL detection
  - `config.rs`: Settings JSON persistence
  - `updater.rs`: yt-dlp version checking and updates
  - `export.rs`: Export history (JSON/CSV), import settings/URL lists
- **yt-dlp Integration** (`ytdlp/`):
  - `binary.rs`: Locate yt-dlp/ffmpeg binaries, version checking
  - `process.rs`: Process spawning (Windows hides console), PID tracking
  - `parser.rs`: Parse yt-dlp stdout for progress (%/speed/ETA/size)

### Key State Flow
1. User pastes URL → `download.ts::fetchVideo()` → `commands::download::fetch_video_info()`
2. Backend runs `yt-dlp --dump-json` → parses JSON → returns VideoInfo
3. User selects format/options → `download()` → `start_download()` (or `start_batch_download()`)
4. Task enters queue (`queue.rs::DownloadQueue`)
5. Queue worker spawns yt-dlp process, emits progress via `download-progress` event
6. Frontend `updateProgress()` updates state, database tracks completion
7. Desktop notifications sent on complete/error (if enabled)

## Important Patterns & Conventions

### Rust Async/Runtime
- Use `tauri::async_runtime::spawn()` in setup, not `tokio::spawn()`
- Queue workers and clipboard watchers run as background async tasks
- Shared state wrapped in `Arc<T>` (Database, ConfigManager, ProcessManager, DownloadQueue)

### Tauri Event System
- Progress updates emitted as `download-progress` events (type: `DownloadProgress`)
- Clipboard URL detection: `clipboard-url` event
- Notifications: `notify` event with `{type, title, body}` payload
- Listeners initialized once in `download.ts::initEventListener()`

### Database Schema (SQLite)
- Table: `download_tasks`
- Fields: id, url, title, thumbnail, format_id, quality_label, audio_only, download_subtitle, subtitle_lang, status, progress, speed, eta, total_size, downloaded_size, output_path, error_message, created_at, completed_at, playlist_title, playlist_index, playlist_total
- Auto-migration: adds playlist fields if missing

### Progress Parsing (`ytdlp/parser.rs`)
- Parses yt-dlp stdout line-by-line (no regex dependency)
- Extracts: percentage, speed, ETA, total size, file path
- Detects merge states: `[Merger]`, `[ffmpeg]`, `[ExtractAudio]`
- Merges output forced to MP4 via `--merge-output-format mp4`

### Binary Locations
Priority order for yt-dlp/ffmpeg:
1. `<app_data_dir>/bin/`
2. Next to executable
3. `binaries/` in repo
4. `CARGO_MANIFEST_DIR/binaries/`
5. System PATH

### Clipboard Monitoring
- Polls every 1.5s via platform-specific commands
- Windows: `powershell Get-Clipboard`
- macOS: `pbpaste`
- Linux: `xclip` or `xsel`
- Filters duplicates, checks against 20+ video site domains

### Settings Persistence
- Stored as JSON in `settings.json` (app_data_dir)
- Loaded at startup, saved immediately on change
- Live updates: max_concurrent changes affect queue, clipboard_watch toggles listener

## Type Safety
- All Tauri commands use TypeScript interfaces (defined in `utils/invoke.ts`)
- Rust structs derive `Serialize`, `Deserialize` for IPC
- Database models in `database/models.rs` with `rusqlite::FromRow` derive

## Known Issues & Notes
- `output_path` may be null on some sites (fallback to download_dir)
- Long filenames truncated in UI (full path in tooltip)
- Windows console hidden via `creation_flags(0x08000000)` for `Command::new()`
- Download queue uses `Notify` for slot availability signaling
- Cancelled tasks use a 5s ignore list to prevent stale progress updates

## Required External Files (Manual Setup)
- `src-tauri/binaries/yt-dlp.exe`: Download from yt-dlp releases
- `src-tauri/binaries/ffmpeg.exe`: ~80MB, required for format merging
- `src-tauri/icons/icon.ico` and `icon.png`: App and tray icons

## Build & Release Targets
- Current: Windows (development)
- Target: Windows/macOS/Linux (requires icons for each platform)
