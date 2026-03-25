<template>
  <Transition name="fade">
    <div v-if="store.videoInfo" class="download-options card">
      <h4 class="section-title">{{ t('options.title') }}</h4>
      <div class="option-row">
        <label class="option-label">{{ t('options.format') }}</label>
        <div class="radio-group">
          <label class="radio-item" :class="{ active: !store.audioOnly }">
            <input type="radio" :value="false" v-model="store.audioOnly" /><span class="radio-icon">🎬</span> {{ t('options.video') }}
          </label>
          <label class="radio-item" :class="{ active: store.audioOnly }">
            <input type="radio" :value="true" v-model="store.audioOnly" /><span class="radio-icon">🎵</span> {{ t('options.audioOnly') }}
          </label>
        </div>
      </div>
      <Transition name="fade">
        <div v-if="!store.audioOnly && !store.isPlaylist" class="option-row">
          <label class="option-label">{{ t('options.quality') }}</label>
          <div class="quality-selector">
            <select class="quality-select" :value="store.selectedFormatId" @change="handleFormatChange">
              <option v-for="f in store.availableFormats" :key="f.format_id" :value="f.format_id">
                {{ f.quality_label }} ({{ f.ext }}) {{ formatFileSize(f.filesize || f.filesize_approx) }}
              </option>
            </select>
          </div>
        </div>
      </Transition>
      <div class="option-row">
        <label class="option-label">{{ t('options.subtitle') }}</label>
        <div class="subtitle-options">
          <label class="checkbox-item"><input type="checkbox" v-model="store.downloadSubtitle" /><span>{{ t('options.downloadSubtitle') }}</span></label>
          <Transition name="fade">
            <select v-if="store.downloadSubtitle" v-model="store.subtitleLang" class="subtitle-lang-select">
              <option value="zh-Hans">{{ t('lang.zhHans') }}</option><option value="en">{{ t('lang.en') }}</option><option value="ja">{{ t('lang.ja') }}</option>
            </select>
          </Transition>
        </div>
      </div>
      <div class="option-row">
        <label class="option-label">{{ t('options.saveTo') }}</label>
        <div class="path-selector">
          <input type="text" class="path-input" :value="downloadPath" readonly @click="selectPath" />
          <button class="btn btn-secondary" @click="selectPath">{{ t('options.browse') }}</button>
        </div>
      </div>
      <div class="download-action">
        <button class="btn btn-primary btn-lg download-btn" @click="handleDownload" :disabled="isDownloading">
          <span v-if="isDownloading" class="spinner"></span><span v-else>⬇️</span>
          {{ downloadButtonText }}
        </button>
      </div>
      <Transition name="fade">
        <div v-if="downloadMessage" class="download-message" :class="downloadMessageType">{{ downloadMessage }}</div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useDownloadStore } from '@/stores/download';
import { useSettingsStore } from '@/stores/settings';
import { t } from '@/utils/i18n';
const store = useDownloadStore();
const settingsStore = useSettingsStore();
const isDownloading = ref(false);
const downloadMessage = ref('');
const downloadMessageType = ref('');
const downloadPath = computed(() => settingsStore.settings.download_path || t('options.selectDir'));
const downloadButtonText = computed(() => {
  if (isDownloading.value) return t('options.starting');
  if (store.isPlaylist && store.selectedEntryCount > 0) return t('options.downloadN', { n: store.selectedEntryCount });
  return t('options.startDownload');
});
function handleFormatChange(e: Event) { const v = (e.target as HTMLSelectElement).value; const f = store.availableFormats.find(x => x.format_id === v); if (f) store.selectFormat(f); }
function formatFileSize(bytes: number | null | undefined): string { if (!bytes) return ''; const units = ['B','KB','MB','GB']; let s = bytes, i = 0; while (s >= 1024 && i < units.length - 1) { s /= 1024; i++; } return `~${s.toFixed(1)} ${units[i]}`; }
async function selectPath() { try { const { open } = await import('@tauri-apps/plugin-dialog'); const s = await open({ directory: true, title: t('options.selectDir') }); if (s && typeof s === 'string') { settingsStore.settings.download_path = s; await settingsStore.save(); } } catch { downloadMessage.value = t('options.selectSaveDir'); downloadMessageType.value = 'warn'; setTimeout(() => { downloadMessage.value = ''; }, 3000); } }
async function handleDownload() {
  if (!settingsStore.settings.download_path) { downloadMessage.value = t('options.selectSaveDir'); downloadMessageType.value = 'warn'; setTimeout(() => { downloadMessage.value = ''; }, 3000); return; }
  if (store.isPlaylist && store.selectedEntryCount === 0) { downloadMessage.value = t('options.selectAtLeast'); downloadMessageType.value = 'warn'; setTimeout(() => { downloadMessage.value = ''; }, 3000); return; }
  isDownloading.value = true; downloadMessage.value = '';
  try { await store.download(); const n = store.isPlaylist ? store.selectedEntryCount : 1; downloadMessage.value = t('options.taskAdded', { n }); downloadMessageType.value = 'success'; setTimeout(() => { downloadMessage.value = ''; }, 3000); }
  catch (e: any) { downloadMessage.value = `❌ ${e.message || t('options.downloadFailed')}`; downloadMessageType.value = 'error'; setTimeout(() => { downloadMessage.value = ''; }, 5000); }
  finally { isDownloading.value = false; }
}
</script>

<style scoped>
.download-options { animation: slideUp 0.3s ease; }
.section-title { font-size: 15px; font-weight: 600; margin-bottom: 16px; color: var(--text-primary); }
.option-row { display: flex; align-items: flex-start; gap: 16px; margin-bottom: 16px; }
.option-label { width: 60px; font-size: 14px; font-weight: 500; color: var(--text-secondary); padding-top: 8px; flex-shrink: 0; }
.radio-group { display: flex; gap: 8px; }
.radio-item { display: flex; align-items: center; gap: 6px; padding: 8px 16px; border: 1px solid var(--border); border-radius: var(--radius-sm); cursor: pointer; transition: var(--transition); font-size: 14px; }
.radio-item:hover { border-color: var(--accent); }
.radio-item.active { border-color: var(--accent); background: var(--accent-light); color: var(--accent); }
.radio-item input { display: none; }
.radio-icon { font-size: 16px; }
.quality-selector { flex: 1; }
.quality-select { width: 100%; padding: 8px 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-primary); font-size: 14px; cursor: pointer; }
.subtitle-options { display: flex; align-items: center; gap: 12px; }
.checkbox-item { display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 14px; }
.subtitle-lang-select { padding: 6px 10px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-primary); font-size: 13px; }
.path-selector { flex: 1; display: flex; gap: 8px; }
.path-input { flex: 1; padding: 8px 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-secondary); font-size: 13px; cursor: pointer; }
.path-input:hover { border-color: var(--accent); }
.download-action { display: flex; justify-content: center; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-light); }
.download-btn { min-width: 200px; }
.download-message { text-align: center; margin-top: 12px; padding: 8px 14px; border-radius: var(--radius-sm); font-size: 14px; font-weight: 500; }
.download-message.success { background: var(--success-msg-bg); color: var(--success-msg-text); }
.download-message.warn { background: var(--warn-msg-bg); color: var(--warn-msg-text); }
.download-message.error { background: var(--error-msg-bg); color: var(--error-msg-text); }
.spinner { width: 16px; height: 16px; border: 2px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: spin 0.6s linear infinite; }
@keyframes slideUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
@keyframes spin { to { transform: rotate(360deg); } }
</style>