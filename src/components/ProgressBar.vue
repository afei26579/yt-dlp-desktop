<template>
  <div class="progress-bar-wrapper">
    <div class="progress-track">
      <div
        class="progress-fill"
        :class="statusClass"
        :style="{ width: `${progress}%` }"
      >
        <div v-if="status === 'Downloading'" class="progress-shine"></div>
      </div>
    </div>
    <span class="progress-text">{{ progress.toFixed(1) }}%</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { DownloadStatus } from '@/utils/invoke';

const props = defineProps<{
  progress: number;
  status: DownloadStatus;
}>();

const statusClass = computed(() => ({
  'fill-downloading': props.status === 'Downloading',
  'fill-merging': props.status === 'Merging',
  'fill-completed': props.status === 'Completed',
  'fill-error': props.status === 'Failed',
  'fill-paused': props.status === 'Paused',
}));
</script>

<style scoped>
.progress-bar-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
}

.progress-track {
  flex: 1;
  height: 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width 0.3s ease;
  position: relative;
  overflow: hidden;
}

.fill-downloading {
  background: linear-gradient(90deg, var(--accent), #748ffc);
}

.fill-merging {
  background: var(--warning);
}

.fill-completed {
  background: var(--success);
}

.fill-error {
  background: var(--error);
}

.fill-paused {
  background: var(--text-tertiary);
}

.progress-shine {
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.3),
    transparent
  );
  animation: shine 2s infinite;
}

@keyframes shine {
  to {
    left: 100%;
  }
}

.progress-text {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  min-width: 48px;
  text-align: right;
}
</style>