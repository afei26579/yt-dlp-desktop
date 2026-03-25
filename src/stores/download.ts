import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type {
  DownloadTask, DownloadProgress, VideoInfo, FormatInfo,
  PlaylistEntry, BatchDownloadEntry,
} from '@/utils/invoke';
import {
  fetchVideoInfo as apiFetchVideoInfo,
  startDownload as apiStartDownload,
  startBatchDownload as apiStartBatchDownload,
  startBatchUrls as apiStartBatchUrls,
  cancelDownload as apiCancelDownload,
  pauseDownload as apiPauseDownload,
  resumeDownload as apiResumeDownload,
  retryDownload as apiRetryDownload,
  getDownloadHistory,
  getQueueStatus,
} from '@/utils/invoke';

export const useDownloadStore = defineStore('download', () => {
  const currentUrl = ref('');
  const videoInfo = ref<VideoInfo | null>(null);
  const isFetching = ref(false);
  const fetchError = ref<string | null>(null);
  const selectedFormatId = ref<string | null>(null);
  const selectedQualityLabel = ref('最佳画质');
  const audioOnly = ref(false);
  const downloadSubtitle = ref(false);
  const subtitleLang = ref('zh-Hans');
  const selectedEntries = ref<Set<number>>(new Set());
  const allEntriesSelected = ref(true);
  const activeTasks = ref<Map<string, DownloadTask>>(new Map());
  const historyTasks = ref<DownloadTask[]>([]);
  const listenerInitialized = ref(false);
  const queueSize = ref(0);
  const activeDownloadCount = ref(0);
  const clipboardDetectedUrl = ref<string | null>(null);
  const batchUrls = ref<string[]>([]);

  // ★ 修复4: 已取消的任务ID集合，忽略后续事件
  const cancelledIds = ref<Set<string>>(new Set());

  const activeTaskList = computed(() => Array.from(activeTasks.value.values()));
  const hasActiveTasks = computed(() => activeTasks.value.size > 0);
  const availableFormats = computed((): FormatInfo[] => videoInfo.value?.formats ?? []);
  const isPlaylist = computed(() => videoInfo.value?.is_playlist ?? false);
  const playlistEntries = computed((): PlaylistEntry[] => videoInfo.value?.entries ?? []);
  const selectedEntryCount = computed(() => selectedEntries.value.size);
  const totalEntryCount = computed(() => playlistEntries.value.length);
  const isBatchMode = computed(() => batchUrls.value.length > 0);

  async function fetchVideo(url: string) {
    if (!url.trim()) return;
    currentUrl.value = url.trim();
    isFetching.value = true;
    fetchError.value = null;
    videoInfo.value = null;
    selectedFormatId.value = null;
    selectedEntries.value = new Set();
    allEntriesSelected.value = true;
    batchUrls.value = [];
    try {
      const info = await apiFetchVideoInfo(url.trim());
      videoInfo.value = info;
      if (info.formats.length > 0) {
        selectedFormatId.value = info.formats[0].format_id;
        selectedQualityLabel.value = info.formats[0].quality_label;
      }
      if (info.is_playlist && info.entries.length > 0) {
        selectedEntries.value = new Set(info.entries.map((_, i) => i));
        allEntriesSelected.value = true;
      }
    } catch (e: any) {
      fetchError.value = typeof e === 'string' ? e : e.message || '解析失败';
    } finally { isFetching.value = false; }
  }

  function setBatchUrls(urls: string[]) {
    batchUrls.value = urls; videoInfo.value = null; fetchError.value = null;
  }

  function toggleEntry(index: number) {
    const s = new Set(selectedEntries.value);
    s.has(index) ? s.delete(index) : s.add(index);
    selectedEntries.value = s;
    allEntriesSelected.value = s.size === playlistEntries.value.length;
  }

  function toggleAllEntries() {
    if (allEntriesSelected.value) {
      selectedEntries.value = new Set();
      allEntriesSelected.value = false;
    } else {
      selectedEntries.value = new Set(playlistEntries.value.map((_, i) => i));
      allEntriesSelected.value = true;
    }
  }

  async function download() {
    if (!videoInfo.value) return;
    if (isPlaylist.value && playlistEntries.value.length > 0) {
      const entries: BatchDownloadEntry[] = playlistEntries.value
        .filter((_, i) => selectedEntries.value.has(i))
        .map(e => ({ url: e.url, title: e.title, thumbnail: e.thumbnail }));
      if (entries.length === 0) throw new Error('请至少选择一个视频');
      const tasks = await apiStartBatchDownload({
        entries, audioOnly: audioOnly.value,
        downloadSubtitle: downloadSubtitle.value,
        subtitleLang: downloadSubtitle.value ? subtitleLang.value : null,
        playlistTitle: videoInfo.value.title,
      });
      for (const task of tasks) activeTasks.value.set(task.id, { ...task, status: 'Queued', progress: 0 });
    } else {
      const task = await apiStartDownload({
        url: currentUrl.value, title: videoInfo.value.title,
        thumbnail: videoInfo.value.thumbnail,
        formatId: audioOnly.value ? null : selectedFormatId.value,
        qualityLabel: audioOnly.value ? 'MP3 音频' : selectedQualityLabel.value,
        audioOnly: audioOnly.value,
        downloadSubtitle: downloadSubtitle.value,
        subtitleLang: downloadSubtitle.value ? subtitleLang.value : null,
      });
      activeTasks.value.set(task.id, { ...task, status: 'Queued', progress: 0 });
    }
    activeTasks.value = new Map(activeTasks.value);
  }

  async function downloadBatchUrls(urls: string[]) {
    if (urls.length === 0) throw new Error('没有有效的链接');
    const tasks = await apiStartBatchUrls({
      urls, audioOnly: audioOnly.value,
      downloadSubtitle: downloadSubtitle.value,
      subtitleLang: downloadSubtitle.value ? subtitleLang.value : null,
    });
    for (const task of tasks) activeTasks.value.set(task.id, { ...task, status: 'Queued', progress: 0 });
    activeTasks.value = new Map(activeTasks.value);
    return tasks.length;
  }

  // ★ 修复4: 取消 — 立刻从界面移除 + 加入忽略列表
  async function cancel(taskId: string) {
    cancelledIds.value.add(taskId);
    activeTasks.value.delete(taskId);
    activeTasks.value = new Map(activeTasks.value);
    try { await apiCancelDownload(taskId); }
    catch (e) { console.error('Cancel failed:', e); }
    // 5秒后清理忽略列表
    setTimeout(() => { cancelledIds.value.delete(taskId); }, 5000);
  }

  // ★ 修复2&3: 暂停 — 立刻更新UI状态
  async function pause(taskId: string) {
    const task = activeTasks.value.get(taskId);
    if (task) {
      activeTasks.value.set(taskId, { ...task, status: 'Paused', speed: null, eta: '暂停中...' });
      activeTasks.value = new Map(activeTasks.value);
    }
    try { await apiPauseDownload(taskId); }
    catch (e) { console.error('Pause failed:', e); }
  }

  // ★ 修复3: 恢复 — 立刻更新UI状态
  async function resume(taskId: string) {
    const task = activeTasks.value.get(taskId);
    if (task) {
      activeTasks.value.set(taskId, { ...task, status: 'Queued', eta: '恢复中...' });
      activeTasks.value = new Map(activeTasks.value);
    }
    try { await apiResumeDownload(taskId); }
    catch (e) { console.error('Resume failed:', e); }
  }

  // ★ 修复6: 重试 — 直接重新入队
  async function retry(taskId: string) {
    try {
      const newTask = await apiRetryDownload(taskId);
      activeTasks.value.set(newTask.id, { ...newTask, status: 'Queued', progress: 0 });
      activeTasks.value = new Map(activeTasks.value);
      // 从历史中移除旧任务
      historyTasks.value = historyTasks.value.filter(t => t.id !== taskId);
    } catch (e: any) {
      console.error('Retry failed:', e);
      throw e;
    }
  }

  async function loadHistory() {
    try { historyTasks.value = await getDownloadHistory(100, 0); } catch (e) { console.error(e); }
  }

  async function refreshQueueStatus() {
    try { const s = await getQueueStatus(); queueSize.value = s.queue_size; activeDownloadCount.value = s.active_count; }
    catch (e) { console.error(e); }
  }

  function updateProgress(progress: DownloadProgress) {
    // ★ 修复4: 忽略已取消任务的事件
    if (cancelledIds.value.has(progress.task_id)) return;

    if (progress.status === 'Paused') {
      const existing = activeTasks.value.get(progress.task_id);
      if (existing) {
        activeTasks.value.set(progress.task_id, {
          ...existing, status: 'Paused', speed: null, eta: '已暂停',
        });
        activeTasks.value = new Map(activeTasks.value);
      }
      return;
    }

    // ★ 修复4: 取消事件也忽略
    if (progress.status === 'Cancelled') {
      activeTasks.value.delete(progress.task_id);
      activeTasks.value = new Map(activeTasks.value);
      return;
    }

    const task = activeTasks.value.get(progress.task_id);
    if (task) {
      const updated: DownloadTask = {
        ...task,
        status: progress.status,
        progress: progress.progress,
        speed: progress.speed ?? task.speed,
        eta: progress.eta ?? task.eta,
        total_size: progress.total_size ?? task.total_size,
        downloaded_size: progress.downloaded_size ?? task.downloaded_size,
        output_path: progress.output_path ?? task.output_path,
        error_message: progress.error_message ?? task.error_message,
      };
      if (progress.status === 'Completed' || progress.status === 'Failed') {
        activeTasks.value.delete(progress.task_id);
        historyTasks.value.unshift(updated);
      } else {
        activeTasks.value.set(progress.task_id, updated);
      }
    } else if (!['Completed', 'Failed', 'Cancelled'].includes(progress.status)) {
      activeTasks.value.set(progress.task_id, {
        id: progress.task_id, url: '', title: '加载中...', thumbnail: null,
        status: progress.status, progress: progress.progress,
        speed: progress.speed, eta: progress.eta,
        total_size: progress.total_size, downloaded_size: progress.downloaded_size,
        output_path: progress.output_path, format_id: null, quality_label: '',
        audio_only: false, download_subtitle: false, subtitle_lang: null,
        error_message: progress.error_message,
        created_at: new Date().toISOString(), completed_at: null,
        playlist_title: null, playlist_index: null, playlist_total: null,
      });
    }
    activeTasks.value = new Map(activeTasks.value);
  }

  function selectFormat(format: FormatInfo) {
    selectedFormatId.value = format.format_id;
    selectedQualityLabel.value = format.quality_label;
  }

  function dismissClipboardUrl() { clipboardDetectedUrl.value = null; }
  function acceptClipboardUrl() {
    if (clipboardDetectedUrl.value) {
      currentUrl.value = clipboardDetectedUrl.value;
      fetchVideo(clipboardDetectedUrl.value);
      clipboardDetectedUrl.value = null;
    }
  }

  async function initEventListener() {
    if (listenerInitialized.value) return;
    await listen<DownloadProgress>('download-progress', (e) => { updateProgress(e.payload); });
    await listen<{ type: string; title: string; body: string }>('notify', async (e) => {
      try {
        const { isPermissionGranted, requestPermission, sendNotification } = await import('@tauri-apps/plugin-notification');
        let granted = await isPermissionGranted();
        if (!granted) { const p = await requestPermission(); granted = p === 'granted'; }
        if (granted) sendNotification({ title: e.payload.title, body: e.payload.body });
      } catch {}
    });
    await listen<string>('clipboard-url', (e) => { clipboardDetectedUrl.value = e.payload; });
    listenerInitialized.value = true;
    setInterval(() => { if (activeTasks.value.size > 0) refreshQueueStatus(); }, 3000);
  }

  function reset() {
    currentUrl.value = ''; videoInfo.value = null; fetchError.value = null;
    selectedFormatId.value = null; audioOnly.value = false; downloadSubtitle.value = false;
    selectedEntries.value = new Set(); allEntriesSelected.value = true; batchUrls.value = [];
  }

  return {
    currentUrl, videoInfo, isFetching, fetchError,
    selectedFormatId, selectedQualityLabel, audioOnly,
    downloadSubtitle, subtitleLang, activeTasks, historyTasks,
    activeTaskList, hasActiveTasks, availableFormats,
    isPlaylist, playlistEntries, selectedEntries, allEntriesSelected,
    selectedEntryCount, totalEntryCount,
    queueSize, activeDownloadCount, clipboardDetectedUrl,
    batchUrls, isBatchMode,
    fetchVideo, setBatchUrls, download, downloadBatchUrls,
    cancel, pause, resume, retry,
    loadHistory, selectFormat, initEventListener, reset,
    toggleEntry, toggleAllEntries, refreshQueueStatus,
    dismissClipboardUrl, acceptClipboardUrl,
  };
});