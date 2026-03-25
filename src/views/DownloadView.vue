<template>
  <div class="download-view">
    <UrlInput @go-settings="(section) => $emit('go-settings', section)" />

    <!-- ★ 自动：有批量URL显示批量面板，否则显示单个流程 -->
    <template v-if="store.isBatchMode">
      <BatchPreview />
    </template>
    <template v-else>
      <VideoPreview />
      <PlaylistSelector />
      <DownloadOptions />
    </template>

    <Transition name="fade">
      <div v-if="store.hasActiveTasks" class="active-downloads">
        <div class="section-header">
          <h4>{{ t('download.active') }}
            <span class="task-counts">
              ({{ downloadingCount }} {{ t('download.downloading') }}
              <template v-if="queuedCount > 0">, {{ queuedCount }} {{ t('download.queued') }}</template>
              <template v-if="pausedCount > 0">, {{ pausedCount }} {{ t('item.paused') }}</template>)
            </span>
          </h4>
        </div>
        <TransitionGroup name="list" tag="div" class="task-list">
          <DownloadItem
            v-for="task in store.activeTaskList"
            :key="task.id"
            :task="task"
            :is-active="true"
            @cancel="store.cancel"
            @pause="store.pause"
            @resume="store.resume"
          />
        </TransitionGroup>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import UrlInput from '@/components/UrlInput.vue';
import VideoPreview from '@/components/VideoPreview.vue';
import PlaylistSelector from '@/components/PlaylistSelector.vue';
import DownloadOptions from '@/components/DownloadOptions.vue';
import BatchPreview from '@/components/BatchPreview.vue';
import DownloadItem from '@/components/DownloadItem.vue';
import { useDownloadStore } from '@/stores/download';
import { t } from '@/utils/i18n';

defineEmits<{ 'go-settings': [section?: string] }>();
const store = useDownloadStore();

const downloadingCount = computed(() =>
  store.activeTaskList.filter(t => t.status === 'Downloading' || t.status === 'Merging' || t.status === 'Pending').length);
const queuedCount = computed(() =>
  store.activeTaskList.filter(t => t.status === 'Queued').length);
const pausedCount = computed(() =>
  store.activeTaskList.filter(t => t.status === 'Paused').length);
</script>

<style scoped>
.download-view { max-width: 720px; margin: 0 auto; }
.active-downloads { margin-top: 24px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.section-header h4 { font-size: 15px; font-weight: 600; color: var(--text-primary); }
.task-counts { font-weight: 400; color: var(--text-secondary); font-size: 13px; }
.task-list { display: flex; flex-direction: column; gap: 8px; }
.list-enter-active, .list-leave-active { transition: all 0.3s ease; }
.list-enter-from { opacity: 0; transform: translateX(-20px); }
.list-leave-to { opacity: 0; transform: translateX(20px); }
</style>