<template>
  <div class="app">
    <TitleBar />
    <ClipboardAlert />
    <nav class="tab-nav">
      <button v-for="tab in tabs" :key="tab.id" class="tab-btn"
        :class="{ active: activeTab === tab.id }" @click="switchTab(tab.id)">
        <span class="tab-icon">{{ tab.icon }}</span>
        <span class="tab-label">{{ t(tab.labelKey) }}</span>
        <span v-if="tab.id === 'queue' && totalActive > 0" class="tab-badge">{{ totalActive }}</span>
      </button>
    </nav>
    <main class="content">
      <Transition name="fade" mode="out-in">
        <DownloadView v-if="activeTab === 'download'" @go-settings="goToSettings" />
        <QueueView v-else-if="activeTab === 'queue'" />
        <SettingsView v-else-if="activeTab === 'settings'" :scroll-to="settingsScrollTarget" />
      </Transition>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import TitleBar from '@/components/TitleBar.vue';
import ClipboardAlert from '@/components/ClipboardAlert.vue';
import DownloadView from '@/views/DownloadView.vue';
import QueueView from '@/views/QueueView.vue';
import SettingsView from '@/views/SettingsView.vue';
import { useDownloadStore } from '@/stores/download';
import { useSettingsStore } from '@/stores/settings';
import { t } from '@/utils/i18n';

const store = useDownloadStore();
const settingsStore = useSettingsStore();
const activeTab = ref('download');
const settingsScrollTarget = ref<string | null>(null);
const totalActive = computed(() => store.activeTaskList.length);

const tabs = [
  { id: 'download', icon: '📥', labelKey: 'tab.download' },
  { id: 'queue', icon: '📋', labelKey: 'tab.queue' },
  { id: 'settings', icon: '⚙️', labelKey: 'tab.settings' },
];

function switchTab(id: string) { activeTab.value = id; if (id !== 'settings') settingsScrollTarget.value = null; }
function goToSettings(section?: string) { settingsScrollTarget.value = section || null; activeTab.value = 'settings'; }

onMounted(async () => {
  await store.initEventListener();
  await settingsStore.load();
  await store.loadHistory();
});
</script>

<style scoped>
.app { height: 100%; display: flex; flex-direction: column; background: var(--bg-primary); }
.tab-nav { display: flex; gap: 2px; padding: 0 16px; background: var(--bg-secondary); border-bottom: 1px solid var(--border-light); flex-shrink: 0; }
.tab-btn { display: flex; align-items: center; gap: 6px; padding: 10px 18px; background: transparent; color: var(--text-secondary); font-size: 14px; font-weight: 500; border: none; border-bottom: 2px solid transparent; cursor: pointer; transition: var(--transition); }
.tab-btn:hover { color: var(--text-primary); background: var(--bg-hover); }
.tab-btn.active { color: var(--accent); border-bottom-color: var(--accent); }
.tab-icon { font-size: 16px; }
.tab-badge { background: var(--accent); color: white; font-size: 11px; padding: 1px 6px; border-radius: var(--radius-full); min-width: 18px; text-align: center; }
.content { flex: 1; overflow-y: auto; padding: 20px; }
</style>