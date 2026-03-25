<template>
  <div class="queue-view">
    <div v-if="store.hasActiveTasks" class="queue-status-panel card">
      <div class="status-grid">
        <div class="status-item"><span class="status-number">{{ downloadingTasks.length }}</span><span class="status-label">{{ t('queue.downloadingLabel') }}</span></div>
        <div class="status-item"><span class="status-number">{{ queuedTasks.length }}</span><span class="status-label">{{ t('queue.queuedLabel') }}</span></div>
        <div class="status-item"><span class="status-number">{{ store.activeTaskList.length }}</span><span class="status-label">{{ t('queue.totalTasks') }}</span></div>
      </div>
    </div>
    <section v-if="downloadingTasks.length > 0" class="section">
      <div class="section-header"><h3>{{ t('queue.downloading') }} ({{ downloadingTasks.length }})</h3></div>
      <div class="task-list"><DownloadItem v-for="task in downloadingTasks" :key="task.id" :task="task" :is-active="true" @cancel="store.cancel" @pause="store.pause" @resume="store.resume" /></div>
    </section>
    <section v-if="queuedTasks.length > 0" class="section">
      <div class="section-header">
        <h3>{{ t('queue.queued') }} ({{ queuedTasks.length }})</h3>
        <button v-if="queuedTasks.length > 1" class="btn btn-ghost" @click="cancelAllQueued">{{ t('queue.cancelAll') }}</button>
      </div>
      <div class="task-list"><DownloadItem v-for="task in queuedTasks" :key="task.id" :task="task" :is-active="true" @cancel="store.cancel" @pause="store.pause" @resume="store.resume" /></div>
    </section>
    <section class="section">
      <div class="section-header">
        <h3>{{ t('queue.history') }} ({{ store.historyTasks.length }})</h3>
        <button v-if="store.historyTasks.length > 0" class="btn btn-ghost" @click="handleClearHistory">{{ t('queue.clearAll') }}</button>
      </div>
      <div v-if="store.historyTasks.length === 0" class="empty-state">
        <div class="empty-icon">📭</div>
        <p>{{ t('queue.empty') }}</p>
        <p class="empty-hint">{{ t('queue.emptyHint') }}</p>
      </div>
      <TransitionGroup v-else name="list" tag="div" class="task-list">
        <DownloadItem v-for="task in store.historyTasks" :key="task.id" :task="task"
          @open-folder="handleOpenFolder" @open-download-dir="handleOpenDownloadDir"
          @retry="handleRetry" @delete="handleDelete" />
      </TransitionGroup>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import DownloadItem from '@/components/DownloadItem.vue';
import { useDownloadStore } from '@/stores/download';
import { useSettingsStore } from '@/stores/settings';
import { openFileLocation, deleteHistoryItem, clearHistory } from '@/utils/invoke';
import { t } from '@/utils/i18n';
import type { DownloadTask } from '@/utils/invoke';

const store = useDownloadStore();
const settingsStore = useSettingsStore();

const downloadingTasks = computed(() =>
  store.activeTaskList.filter(item =>
    item.status === 'Downloading' || item.status === 'Merging'
    || item.status === 'Pending' || item.status === 'Fetching'
    || item.status === 'Paused' 
  )
);
const queuedTasks = computed(() =>
  store.activeTaskList.filter(item => item.status === 'Queued')
);

async function cancelAllQueued() {
  for (const task of queuedTasks.value) { await store.cancel(task.id); }
}
async function handleOpenFolder(path: string) {
  try { await openFileLocation(path); } catch (e) { console.error(e); }
}
async function handleOpenDownloadDir() {
  try { const p = settingsStore.settings.download_path; if (p) await openFileLocation(p); } catch (e) { console.error(e); }
}
async function handleRetry(task: DownloadTask) {
  try {
    await store.retry(task.id);
  } catch (e) {
    console.error('Retry failed:', e);
    // 降级：跳到下载页重新解析
    store.currentUrl = task.url;
    store.fetchVideo(task.url);
  }
}
async function handleDelete(id: string) {
  try { await deleteHistoryItem(id); store.historyTasks = store.historyTasks.filter(item => item.id !== id); } catch (e) { console.error(e); }
}
async function handleClearHistory() {
  if (confirm(t('queue.clearConfirm'))) {
    try { await clearHistory(); store.historyTasks = []; } catch (e) { console.error(e); }
  }
}
</script>

<style scoped>
.queue-view { max-width: 720px; margin: 0 auto; }
.queue-status-panel { margin-bottom: 20px; padding: 16px; }
.status-grid { display: flex; justify-content: space-around; text-align: center; }
.status-item { display: flex; flex-direction: column; gap: 4px; }
.status-number { font-size: 28px; font-weight: 700; color: var(--accent); }
.status-label { font-size: 13px; color: var(--text-secondary); }
.section { margin-bottom: 32px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.section-header h3 { font-size: 16px; font-weight: 600; }
.task-list { display: flex; flex-direction: column; gap: 4px; }
.empty-state { text-align: center; padding: 60px 20px; color: var(--text-tertiary); }
.empty-icon { font-size: 48px; margin-bottom: 12px; }
.empty-state p { font-size: 15px; margin-bottom: 4px; }
.empty-hint { font-size: 13px !important; }
.list-enter-active, .list-leave-active { transition: all 0.3s ease; }
.list-enter-from, .list-leave-to { opacity: 0; transform: translateY(-10px); }
</style>