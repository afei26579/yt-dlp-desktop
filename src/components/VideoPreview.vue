<template>
  <Transition name="fade">
    <div v-if="store.videoInfo" class="video-preview card">
      <div class="preview-content">
        <div class="thumbnail-wrapper">
          <img v-if="thumbnailUrl" :src="thumbnailUrl" :alt="store.videoInfo.title" class="thumbnail" referrerpolicy="no-referrer" @error="imgError = true" />
          <div v-else class="thumbnail-placeholder">{{ store.videoInfo.is_playlist ? '📃' : '🎬' }}</div>
          <span v-if="formattedDuration" class="duration-badge">{{ formattedDuration }}</span>
          <span v-if="store.videoInfo.is_playlist" class="playlist-count-badge">{{ store.videoInfo.playlist_count }} {{ t('preview.videos') }}</span>
        </div>
        <div class="video-info">
          <h3 class="video-title" :title="store.videoInfo.title">{{ store.videoInfo.title }}</h3>
          <div class="video-meta">
            <span v-if="store.videoInfo.uploader" class="meta-item">👤 {{ store.videoInfo.uploader }}</span>
            <span v-if="formattedDate" class="meta-item">📅 {{ formattedDate }}</span>
            <span v-if="formattedDuration" class="meta-item">⏱ {{ formattedDuration }}</span>
          </div>
          <div v-if="store.videoInfo.is_playlist" class="playlist-badge">📃 {{ t('preview.playlist') }} · {{ store.videoInfo.playlist_count }} {{ t('preview.videos') }}</div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useDownloadStore } from '@/stores/download';
import { t } from '@/utils/i18n';
const store = useDownloadStore();
const imgError = ref(false);
watch(() => store.videoInfo, () => { imgError.value = false; });
const thumbnailUrl = computed(() => { if (imgError.value) return null; const u = store.videoInfo?.thumbnail; if (!u) return null; return u.startsWith('http://') ? u.replace('http://', 'https://') : u; });
const formattedDuration = computed(() => { const d = store.videoInfo?.duration; if (!d || d <= 0) return ''; const h = Math.floor(d / 3600), m = Math.floor((d % 3600) / 60), s = Math.floor(d % 60); return h > 0 ? `${h}:${m.toString().padStart(2,'0')}:${s.toString().padStart(2,'0')}` : `${m}:${s.toString().padStart(2,'0')}`; });
const formattedDate = computed(() => { const d = store.videoInfo?.upload_date; if (!d) return ''; return d.length === 8 ? `${d.slice(0,4)}-${d.slice(4,6)}-${d.slice(6,8)}` : d; });
</script>

<style scoped>
.video-preview { margin-bottom: 16px; animation: slideUp 0.3s ease; }
@keyframes slideUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
.preview-content { display: flex; gap: 16px; }
.thumbnail-wrapper { position: relative; flex-shrink: 0; width: 200px; height: 112px; border-radius: var(--radius-sm); overflow: hidden; background: var(--bg-tertiary); }
.thumbnail { width: 100%; height: 100%; object-fit: cover; }
.thumbnail-placeholder { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 36px; background: var(--bg-tertiary); }
.duration-badge { position: absolute; bottom: 6px; right: 6px; background: rgba(0,0,0,0.8); color: white; font-size: 12px; font-weight: 600; padding: 2px 8px; border-radius: 4px; font-family: 'Consolas', monospace; }
.playlist-count-badge { position: absolute; top: 6px; right: 6px; background: rgba(0,0,0,0.75); color: white; font-size: 11px; font-weight: 600; padding: 3px 8px; border-radius: 4px; }
.video-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 8px; }
.video-title { font-size: 16px; font-weight: 600; line-height: 1.4; color: var(--text-primary); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.video-meta { display: flex; gap: 16px; flex-wrap: wrap; }
.meta-item { font-size: 13px; color: var(--text-secondary); white-space: nowrap; }
.playlist-badge { display: inline-flex; align-items: center; gap: 4px; background: var(--accent-light); color: var(--accent); font-size: 13px; font-weight: 500; padding: 4px 10px; border-radius: var(--radius-full); width: fit-content; }
</style>