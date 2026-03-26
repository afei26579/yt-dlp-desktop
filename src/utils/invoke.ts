import { invoke } from '@tauri-apps/api/core';

export interface VideoInfo {
  id: string;
  title: string;
  url: string;
  thumbnail: string | null;
  duration: number | null;
  uploader: string | null;
  upload_date: string | null;
  description: string | null;
  webpage_url: string;
  formats: FormatInfo[];
  is_playlist: boolean;
  playlist_count: number | null;
  entries: PlaylistEntry[];
  available_subtitles: string[]; // 可用的字幕语言列表
  has_subtitles: boolean; // 是否有字幕
}

export interface PlaylistEntry {
  id: string;
  title: string;
  url: string;
  thumbnail: string | null;
  duration: number | null;
  uploader: string | null;
  index: number;
}

export interface FormatInfo {
  format_id: string;
  format_note: string | null;
  ext: string;
  resolution: string | null;
  filesize: number | null;
  filesize_approx: number | null;
  vcodec: string | null;
  acodec: string | null;
  quality_label: string;
}

export interface DownloadTask {
  id: string;
  url: string;
  title: string;
  thumbnail: string | null;
  status: DownloadStatus;
  progress: number;
  speed: string | null;
  eta: string | null;
  total_size: string | null;
  downloaded_size: string | null;
  output_path: string | null;
  format_id: string | null;
  quality_label: string;
  audio_only: boolean;
  download_subtitle: boolean;
  subtitle_lang: string | null;
  error_message: string | null;
  created_at: string;
  completed_at: string | null;
  playlist_title: string | null;
  playlist_index: number | null;
  playlist_total: number | null;
}



export interface DownloadProgress {
  task_id: string;
  status: DownloadStatus;
  progress: number;
  speed: string | null;
  eta: string | null;
  total_size: string | null;
  downloaded_size: string | null;
  output_path: string | null;
  error_message: string | null;
}

export interface AppSettings {
  download_path: string;
  max_concurrent: number;
  filename_template: string;
  proxy_mode: 'None' | 'System' | 'Custom';
  proxy_url: string | null;
  use_browser_cookie: boolean;
  browser_type: string;
  cookie_file_path: string | null;
  auto_check_update: boolean;
  minimize_to_tray: boolean;
  clipboard_watch: boolean;
  theme: string;
  language: string;
  extra_args: string | null;
  notify_on_complete: boolean;
  notify_on_error: boolean;
  speed_limit: string | null;
  download_thumbnail: boolean;
  download_metadata: boolean;
  audio_quality: string;
}

export interface QueueStatus {
  queue_size: number;
  active_count: number;
}

export interface BatchDownloadEntry {
  url: string;
  title: string;
  thumbnail: string | null;
}

// ===== API 函数 =====

export async function fetchVideoInfo(url: string): Promise<VideoInfo> {
  return invoke('fetch_video_info', { url });
}

export async function startDownload(params: {
  url: string;
  title: string;
  thumbnail: string | null;
  formatId: string | null;
  qualityLabel: string;
  audioOnly: boolean;
  downloadSubtitle: boolean;
  subtitleLang: string | null;
  playlistTitle?: string | null;
  playlistIndex?: number | null;
  playlistTotal?: number | null;
}): Promise<DownloadTask> {
  return invoke('start_download', {
    url: params.url,
    title: params.title,
    thumbnail: params.thumbnail,
    formatId: params.formatId,
    qualityLabel: params.qualityLabel,
    audioOnly: params.audioOnly,
    downloadSubtitle: params.downloadSubtitle,
    subtitleLang: params.subtitleLang,
    playlistTitle: params.playlistTitle ?? null,
    playlistIndex: params.playlistIndex ?? null,
    playlistTotal: params.playlistTotal ?? null,
  });
}

export async function startBatchDownload(params: {
  entries: BatchDownloadEntry[];
  audioOnly: boolean;
  downloadSubtitle: boolean;
  subtitleLang: string | null;
  playlistTitle: string;
}): Promise<DownloadTask[]> {
  return invoke('start_batch_download', {
    entries: params.entries,
    audioOnly: params.audioOnly,
    downloadSubtitle: params.downloadSubtitle,
    subtitleLang: params.subtitleLang,
    playlistTitle: params.playlistTitle,
  });
}

export async function cancelDownload(taskId: string): Promise<void> {
  return invoke('cancel_download', { taskId });
}

export async function getDownloadHistory(limit: number = 50, offset: number = 0): Promise<DownloadTask[]> {
  return invoke('get_download_history', { limit, offset });
}

export async function clearHistory(): Promise<void> {
  return invoke('clear_history');
}

export async function deleteHistoryItem(id: string): Promise<void> {
  return invoke('delete_history_item', { id });
}

export async function openFileLocation(path: string): Promise<void> {
  return invoke('open_file_location', { path });
}

export async function checkYtdlp(): Promise<string> {
  return invoke('check_ytdlp');
}

export async function updateYtdlp(): Promise<string> {
  return invoke('update_ytdlp');
}

export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings });
}

export async function getQueueStatus(): Promise<QueueStatus> {
  return invoke('get_queue_status');
}

export async function updateMaxConcurrent(max: number): Promise<void> {
  return invoke('update_max_concurrent', { max });
}

export async function setClipboardWatch(enabled: boolean): Promise<void> {
  return invoke('set_clipboard_watch', { enabled });
}

export async function exportHistory(format: string, path: string): Promise<void> {
  return invoke('export_history', { format, path });
}

export async function importUrls(path: string): Promise<string[]> {
  return invoke('import_urls', { path });
}

export async function exportSettingsFile(path: string): Promise<void> {
  return invoke('export_settings_file', { path });
}

export async function importSettingsFile(path: string): Promise<AppSettings> {
  return invoke('import_settings_file', { path });
}
export async function pauseDownload(taskId: string): Promise<void> {
  return invoke('pause_download', { taskId });
}

export async function resumeDownload(taskId: string): Promise<void> {
  return invoke('resume_download', { taskId });
}

export async function startBatchUrls(params: {
  urls: string[];
  audioOnly: boolean;
  downloadSubtitle: boolean;
  subtitleLang: string | null;
}): Promise<DownloadTask[]> {
  return invoke('start_batch_urls', {
    urls: params.urls,
    audioOnly: params.audioOnly,
    downloadSubtitle: params.downloadSubtitle,
    subtitleLang: params.subtitleLang,
  });
}
export async function retryDownload(taskId: string): Promise<DownloadTask> {
  return invoke('retry_download', { taskId });
}
export interface VideoInfo {
  id: string;
  title: string;
  url: string;
  thumbnail: string | null;
  duration: number | null;
  uploader: string | null;
  upload_date: string | null;
  description: string | null;
  webpage_url: string;
  formats: FormatInfo[];
  is_playlist: boolean;
  playlist_count: number | null;
}

export interface FormatInfo {
  format_id: string;
  format_note: string | null;
  ext: string;
  resolution: string | null;
  filesize: number | null;
  filesize_approx: number | null;
  vcodec: string | null;
  acodec: string | null;
  quality_label: string;
}

export interface DownloadTask {
  id: string;
  url: string;
  title: string;
  thumbnail: string | null;
  status: DownloadStatus;
  progress: number;
  speed: string | null;
  eta: string | null;
  total_size: string | null;
  downloaded_size: string | null;
  output_path: string | null;
  format_id: string | null;
  quality_label: string;
  audio_only: boolean;
  download_subtitle: boolean;
  subtitle_lang: string | null;
  error_message: string | null;
  created_at: string;
  completed_at: string | null;
}

export type DownloadStatus =
  | 'Queued'
  | 'Pending'
  | 'Fetching'
  | 'Downloading'
  | 'Paused'
  | 'Merging'
  | 'Completed'
  | 'Failed'
  | 'Cancelled';

export interface DownloadProgress {
  task_id: string;
  status: DownloadStatus;
  progress: number;
  speed: string | null;
  eta: string | null;
  total_size: string | null;
  downloaded_size: string | null;
  output_path: string | null;
  error_message: string | null;
}

export interface AppSettings {
  download_path: string;
  max_concurrent: number;
  filename_template: string;
  proxy_mode: 'None' | 'System' | 'Custom';
  proxy_url: string | null;
  use_browser_cookie: boolean;
  browser_type: string;
  cookie_file_path: string | null;
  auto_check_update: boolean;
  minimize_to_tray: boolean;
  clipboard_watch: boolean;
  theme: string;
  language: string;
  extra_args: string | null;
}

export interface CookieDbInfo {
  path: string;
  exists: boolean;
  size: number;
}

export interface CookieDiagnostic {
  method: string;
  browser_type: string;
  cookie_file_path: string | null;
  cookie_file_exists: boolean;
  cookie_file_size: number;
  browser_running: boolean;
  cookie_db_found: boolean;
  cookie_db_paths: CookieDbInfo[];
  test_result: string | null;
  suggestions: string[];
}

export async function diagnoseCookie(): Promise<CookieDiagnostic> {
  return invoke('diagnose_cookie');
}