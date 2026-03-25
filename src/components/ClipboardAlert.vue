<template>
  <Transition name="slide-down">
    <div v-if="store.clipboardDetectedUrl" class="clipboard-alert">
      <div class="alert-content">
        <span class="alert-icon">📋</span>
        <div class="alert-text">
          <p class="alert-title">{{ t('clipboard.detected') }}</p>
          <p class="alert-url">{{ store.clipboardDetectedUrl }}</p>
        </div>
        <div class="alert-actions">
          <button class="btn btn-primary btn-sm" @click="store.acceptClipboardUrl">{{ t('clipboard.fetch') }}</button>
          <button class="btn btn-ghost btn-sm" @click="store.dismissClipboardUrl">✕</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useDownloadStore } from '@/stores/download';
import { t } from '@/utils/i18n';
const store = useDownloadStore();
</script>

<style scoped>
.clipboard-alert { position: fixed; top: 44px; left: 50%; transform: translateX(-50%); z-index: 1000; min-width: 400px; max-width: 600px; }
.alert-content { display: flex; align-items: center; gap: 12px; padding: 12px 16px; background: var(--bg-card); border: 1px solid var(--accent); border-radius: var(--radius-md); box-shadow: var(--shadow-md); }
.alert-icon { font-size: 24px; flex-shrink: 0; }
.alert-text { flex: 1; min-width: 0; }
.alert-title { font-size: 14px; font-weight: 600; color: var(--text-primary); margin-bottom: 2px; }
.alert-url { font-size: 12px; color: var(--accent); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.alert-actions { display: flex; gap: 6px; flex-shrink: 0; }
.btn-sm { padding: 5px 12px; font-size: 12px; }
.slide-down-enter-active { animation: slideDown 0.3s ease; }
.slide-down-leave-active { animation: slideDown 0.3s ease reverse; }
@keyframes slideDown { from { opacity: 0; transform: translateX(-50%) translateY(-20px); } to { opacity: 1; transform: translateX(-50%) translateY(0); } }
</style>