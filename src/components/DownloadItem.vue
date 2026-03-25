<template>
  <div class="download-item" :class="{ 'is-active': isActive }">
    <div class="item-thumb">
      <img v-if="task.thumbnail" :src="task.thumbnail" alt="" referrerpolicy="no-referrer"
        @error="($event.target as HTMLImageElement).style.display='none'" />
      <div v-else class="thumb-placeholder">🎬</div>
    </div>
    <div class="item-info">
      <div class="item-title-row">
        <div class="item-title" :title="task.title">{{ task.title }}</div>
        <span v-if="task.playlist_title" class="playlist-tag">📃 {{ task.playlist_index }}/{{ task.playlist_total }}</span>
      </div>

      <template v-if="isActive">
        <div v-if="task.status === 'Queued'" class="queued-status">
          <span>⏳</span><span class="queued-text">{{ t('item.queued') }}</span>
        </div>

        <div v-else-if="task.status === 'Paused'" class="paused-status">
          <ProgressBar :progress="task.progress" :status="task.status" />
          <div class="paused-hint">{{ t('item.pausedHint') }}</div>
        </div>

        <template v-else>
          <ProgressBar :progress="task.progress" :status="task.status" />
          <div class="item-stats">
            <span v-if="task.status === 'Downloading'" class="stat">📶 {{ task.speed || t('item.calculating') }}</span>
            <span v-if="task.eta" class="stat">⏱ {{ task.eta }}</span>
            <span v-if="task.total_size" class="stat">
              📦 {{ task.downloaded_size ? `${task.downloaded_size} / ` : '' }}{{ task.total_size }}
            </span>
            <span v-if="task.status === 'Merging'" class="stat merging">{{ t('item.merging') }}</span>
            <span v-if="task.status === 'Pending'" class="stat">{{ t('item.preparing') }}</span>
            <!-- ★ 友好错误 -->
            <span v-if="task.status === 'Failed'" class="stat error">❌ {{ displayError }}</span>
          </div>
        </template>
      </template>

      <template v-else>
        <div class="item-meta">
          <span class="status-badge" :class="statusBadgeClass">{{ statusIcon }} {{ statusText }}</span>
          <span class="meta-text">{{ task.quality_label }}</span>
          <span class="meta-text">{{ formatTime(task.created_at) }}</span>
        </div>
        <!-- ★ 历史记录中的错误也友好化 -->
        <div v-if="task.status === 'Failed' && task.error_message" class="error-detail">
          <div class="error-friendly">{{ displayError }}</div>
          <div v-if="isRetryable" class="error-retry-hint">
            💡 {{ t('error.solutionRetryLater') }}
          </div>
        </div>
        <div v-if="task.status === 'Completed' && task.output_path" class="file-path" :title="task.output_path">
          📄 {{ shortenPath(task.output_path) }}
        </div>
      </template>
    </div>

    <div class="item-actions">
      <template v-if="isActive">
        <button v-if="task.status === 'Downloading' || task.status === 'Merging'"
          class="btn btn-ghost action-btn" @click="$emit('pause', task.id)" :title="t('item.pause')">⏸</button>
        <button v-if="task.status === 'Paused'"
          class="btn btn-ghost action-btn resume-btn" @click="$emit('resume', task.id)" :title="t('item.resume')">▶️</button>
        <button class="btn btn-ghost action-btn" @click="$emit('cancel', task.id)" :title="t('item.cancel')">✕</button>
      </template>
      <template v-else>
        <button v-if="task.status === 'Completed' && task.output_path" class="btn btn-ghost action-btn"
          @click="$emit('open-folder', task.output_path)" :title="t('item.openFolder')">📂</button>
        <button v-if="task.status === 'Completed' && !task.output_path" class="btn btn-ghost action-btn"
          @click="$emit('open-download-dir')" :title="t('item.openDownloadDir')">📁</button>
        <button v-if="task.status === 'Failed'" class="btn btn-ghost action-btn"
          @click="$emit('retry', task)" :title="t('item.retry')">🔄</button>
        <button class="btn btn-ghost action-btn" @click="$emit('delete', task.id)" :title="t('item.deleteRecord')">🗑</button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import ProgressBar from './ProgressBar.vue';
import type { DownloadTask } from '@/utils/invoke';
import { t } from '@/utils/i18n';
import { friendlyDownloadError, isRetryableError } from '@/utils/errors';

const props = defineProps<{ task: DownloadTask; isActive?: boolean }>();

defineEmits<{
  cancel: [id: string];
  pause: [id: string];
  resume: [id: string];
  'open-folder': [path: string];
  'open-download-dir': [];
  retry: [task: DownloadTask];
  delete: [id: string];
}>();

// ★ 友好化错误消息
const displayError = computed(() =>
  friendlyDownloadError(props.task.error_message || '')
);

// ★ 是否可重试（网络错误）
const isRetryable = computed(() =>
  isRetryableError(props.task.error_message || '')
);

const statusIcon = computed(() => ({ Completed: '✅', Failed: '❌', Cancelled: '⏹' }[props.task.status] || '⏳'));
const statusText = computed(() => ({
  Completed: t('item.completed'), Failed: t('item.failed'), Cancelled: t('item.cancelled'),
}[props.task.status] || props.task.status));
const statusBadgeClass = computed(() => ({
  'badge-success': props.task.status === 'Completed',
  'badge-error': props.task.status === 'Failed',
  'badge-muted': props.task.status === 'Cancelled',
}));

function formatTime(iso: string): string {
  try { const d = new Date(iso), n = new Date();
    return d.toDateString() === n.toDateString()
      ? d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
      : d.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' });
  } catch { return ''; }
}

function shortenPath(p: string): string {
  const parts = p.replace(/\\/g, '/').split('/');
  return parts.length <= 2 ? p : '.../' + parts[parts.length - 1];
}
</script>

<style scoped>
.download-item { display: flex; align-items: center; gap: 12px; padding: 12px; border-radius: var(--radius-sm); transition: var(--transition); }
.download-item:hover { background: var(--bg-hover); }
.download-item.is-active { background: var(--bg-secondary); border: 1px solid var(--border-light); border-radius: var(--radius-md); }
.item-thumb { width: 64px; height: 36px; border-radius: 4px; overflow: hidden; flex-shrink: 0; background: var(--bg-tertiary); }
.item-thumb img { width: 100%; height: 100%; object-fit: cover; }
.thumb-placeholder { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 20px; }
.item-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 6px; }
.item-title-row { display: flex; align-items: center; gap: 8px; }
.item-title { font-size: 14px; font-weight: 500; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex: 1; min-width: 0; }
.playlist-tag { flex-shrink: 0; font-size: 11px; font-weight: 500; color: var(--accent); background: var(--accent-light); padding: 2px 8px; border-radius: var(--radius-full); }
.queued-status { display: flex; align-items: center; gap: 6px; padding: 4px 0; }
.queued-text { font-size: 13px; color: var(--text-secondary); font-weight: 500; }
.paused-status { display: flex; flex-direction: column; gap: 4px; }
.paused-hint { font-size: 12px; color: var(--warning); font-weight: 500; }
.item-stats { display: flex; gap: 12px; flex-wrap: wrap; }
.stat { font-size: 12px; color: var(--text-secondary); }
.stat.merging { color: var(--warning); font-weight: 500; }
.stat.error { color: var(--error); }
.item-meta { display: flex; align-items: center; gap: 10px; }
.status-badge { font-size: 12px; font-weight: 500; padding: 2px 8px; border-radius: var(--radius-full); }
.badge-success { background: var(--badge-success-bg); color: var(--badge-success-text); }
.badge-error { background: var(--badge-error-bg); color: var(--badge-error-text); }
.badge-muted { background: var(--bg-tertiary); color: var(--text-tertiary); }
.meta-text { font-size: 12px; color: var(--text-tertiary); }
.file-path { font-size: 12px; color: var(--text-tertiary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

/* ★ 错误详情美化 */
.error-detail {
  font-size: 12px;
  background: var(--error-detail-bg);
  padding: 8px 12px;
  border-radius: 4px;
  max-height: 80px;
  overflow-y: auto;
}
.error-friendly {
  color: var(--error);
  font-weight: 500;
  line-height: 1.5;
}
.error-retry-hint {
  margin-top: 4px;
  color: var(--text-secondary);
  font-size: 11px;
}

.item-actions { display: flex; gap: 2px; flex-shrink: 0; }
.action-btn { font-size: 16px; width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; border-radius: var(--radius-sm); }
.action-btn:hover { transform: scale(1.1); }
.resume-btn { color: var(--success); }
.resume-btn:hover { background: var(--success); color: white; }
</style>