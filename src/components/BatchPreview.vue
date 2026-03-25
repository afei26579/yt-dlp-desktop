<template>
  <div v-if="store.isBatchMode" class="batch-preview card">
    <div class="batch-header">
      <h4 class="batch-title">📋 {{ t('batch.title') }}</h4>
      <span class="batch-count">{{ store.batchUrls.length }} {{ t('preview.videos') }}</span>
    </div>

    <div class="url-list">
      <div v-for="(u, i) in store.batchUrls" :key="i" class="url-row">
        <span class="url-index">{{ i + 1 }}</span>
        <span class="url-text" :title="u">{{ u }}</span>
      </div>
    </div>

    <!-- 下载选项 -->
    <div class="batch-options">
      <div class="option-row">
        <label class="option-label">{{ t('options.format') }}</label>
        <div class="radio-group">
          <label class="radio-item" :class="{ active: !store.audioOnly }">
            <input type="radio" :value="false" v-model="store.audioOnly" />
            <span class="radio-icon">🎬</span> {{ t('options.video') }}
          </label>
          <label class="radio-item" :class="{ active: store.audioOnly }">
            <input type="radio" :value="true" v-model="store.audioOnly" />
            <span class="radio-icon">🎵</span> {{ t('options.audioOnly') }}
          </label>
        </div>
      </div>

      <div class="option-row">
        <label class="option-label">{{ t('options.saveTo') }}</label>
        <div class="path-selector">
          <input type="text" class="path-input" :value="downloadPath" readonly @click="selectPath" />
          <button class="btn btn-secondary" @click="selectPath">{{ t('options.browse') }}</button>
        </div>
      </div>

      <button
        class="btn btn-primary btn-lg batch-btn"
        @click="handleBatchDownload"
        :disabled="isDownloading"
      >
        <span v-if="isDownloading" class="spinner"></span>
        <span v-else>⬇️</span>
        {{ isDownloading ? t('options.starting') : t('batch.startAll', { n: store.batchUrls.length }) }}
      </button>

      <Transition name="fade">
        <div v-if="msg" class="batch-msg" :class="msgType">{{ msg }}</div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useDownloadStore } from '@/stores/download';
import { useSettingsStore } from '@/stores/settings';
import { t } from '@/utils/i18n';

const store = useDownloadStore();
const settingsStore = useSettingsStore();
const isDownloading = ref(false);
const msg = ref('');
const msgType = ref('');

const downloadPath = computed(() => settingsStore.settings.download_path || t('options.selectDir'));

async function selectPath() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const s = await open({ directory: true, title: t('options.selectDir') });
    if (s && typeof s === 'string') { settingsStore.settings.download_path = s; await settingsStore.save(); }
  } catch {}
}

async function handleBatchDownload() {
  if (!settingsStore.settings.download_path) {
    msg.value = t('options.selectSaveDir'); msgType.value = 'warn';
    setTimeout(() => { msg.value = ''; }, 3000); return;
  }
  isDownloading.value = true; msg.value = '';
  try {
    const count = await store.downloadBatchUrls(store.batchUrls);
    msg.value = t('options.taskAdded', { n: count }); msgType.value = 'success';
    store.reset(); // 清除批量URL
    setTimeout(() => { msg.value = ''; }, 3000);
  } catch (e: any) {
    msg.value = `❌ ${e.message || t('options.downloadFailed')}`; msgType.value = 'error';
    setTimeout(() => { msg.value = ''; }, 5000);
  } finally { isDownloading.value = false; }
}
</script>

<style scoped>
.batch-preview { margin-bottom: 16px; animation: slideUp 0.3s ease; }
@keyframes slideUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

.batch-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 10px; border-bottom: 1px solid var(--border-light); }
.batch-title { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.batch-count { font-size: 13px; font-weight: 600; color: var(--accent); background: var(--accent-light); padding: 3px 10px; border-radius: var(--radius-full); }

.url-list { max-height: 240px; overflow-y: auto; margin-bottom: 16px; }
.url-row { display: flex; align-items: center; gap: 10px; padding: 6px 8px; border-radius: var(--radius-sm); }
.url-row:nth-child(odd) { background: var(--bg-secondary); }
.url-index { width: 24px; text-align: center; font-size: 12px; font-weight: 600; color: var(--text-tertiary); flex-shrink: 0; }
.url-text { flex: 1; font-size: 12px; color: var(--text-secondary); font-family: 'Consolas', monospace; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

.batch-options { border-top: 1px solid var(--border-light); padding-top: 16px; }
.option-row { display: flex; align-items: flex-start; gap: 16px; margin-bottom: 16px; }
.option-label { width: 60px; font-size: 14px; font-weight: 500; color: var(--text-secondary); padding-top: 8px; flex-shrink: 0; }
.radio-group { display: flex; gap: 8px; }
.radio-item { display: flex; align-items: center; gap: 6px; padding: 8px 16px; border: 1px solid var(--border); border-radius: var(--radius-sm); cursor: pointer; transition: var(--transition); font-size: 14px; }
.radio-item:hover { border-color: var(--accent); }
.radio-item.active { border-color: var(--accent); background: var(--accent-light); color: var(--accent); }
.radio-item input { display: none; }
.radio-icon { font-size: 16px; }
.path-selector { flex: 1; display: flex; gap: 8px; }
.path-input { flex: 1; padding: 8px 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-secondary); font-size: 13px; cursor: pointer; }

.batch-btn { width: 100%; display: flex; align-items: center; justify-content: center; gap: 8px; }

.batch-msg { text-align: center; margin-top: 12px; padding: 8px 14px; border-radius: var(--radius-sm); font-size: 14px; font-weight: 500; }
.batch-msg.success { background: var(--success-msg-bg); color: var(--success-msg-text); }
.batch-msg.warn { background: var(--warn-msg-bg); color: var(--warn-msg-text); }
.batch-msg.error { background: var(--error-msg-bg); color: var(--error-msg-text); }

.spinner { width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: spin 0.6s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>