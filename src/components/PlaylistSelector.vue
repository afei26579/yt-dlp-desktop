<template>
  <div v-if="store.isPlaylist && store.playlistEntries.length > 0" class="playlist-selector card">
    <div class="playlist-header">
      <div class="header-left">
        <h4 class="section-title">{{ t('playlist.content') }}</h4>
        <span class="count-badge">{{ store.selectedEntryCount }} / {{ store.totalEntryCount }} {{ t('playlist.selected') }}</span>
      </div>
      <button class="btn btn-ghost" @click="store.toggleAllEntries">
        {{ store.allEntriesSelected ? t('playlist.deselectAll') : t('playlist.selectAll') }}
      </button>
    </div>
    <div class="entry-list">
      <div v-for="(entry, index) in store.playlistEntries" :key="entry.id || index"
        class="entry-item" :class="{ selected: store.selectedEntries.has(index) }" @click="store.toggleEntry(index)">
        <div class="entry-checkbox"><input type="checkbox" :checked="store.selectedEntries.has(index)" @click.stop @change="store.toggleEntry(index)" /></div>
        <div class="entry-index">{{ entry.index }}</div>
        <div class="entry-thumb">
          <img v-if="entry.thumbnail" :src="entry.thumbnail" alt="" referrerpolicy="no-referrer" @error="($event.target as HTMLImageElement).style.display='none'" />
          <div v-else class="thumb-placeholder">🎬</div>
        </div>
        <div class="entry-info">
          <div class="entry-title" :title="entry.title">{{ entry.title }}</div>
          <div class="entry-meta">
            <span v-if="entry.uploader" class="meta">{{ entry.uploader }}</span>
            <span v-if="entry.duration" class="meta">{{ formatDuration(entry.duration) }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="store.playlistEntries.length > 10" class="playlist-footer">
      <span class="footer-text">{{ t('playlist.total', { total: store.totalEntryCount, selected: store.selectedEntryCount }) }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDownloadStore } from '@/stores/download';
import { t } from '@/utils/i18n';
const store = useDownloadStore();
function formatDuration(s: number | null): string { if (!s || s <= 0) return ''; const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = Math.floor(s % 60); return h > 0 ? `${h}:${m.toString().padStart(2,'0')}:${sec.toString().padStart(2,'0')}` : `${m}:${sec.toString().padStart(2,'0')}`; }
</script>

<style scoped>
.playlist-selector { margin-bottom: 16px; animation: slideUp 0.3s ease; }
@keyframes slideUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
.playlist-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; padding-bottom: 10px; border-bottom: 1px solid var(--border-light); }
.header-left { display: flex; align-items: center; gap: 10px; }
.section-title { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.count-badge { background: var(--accent-light); color: var(--accent); font-size: 12px; font-weight: 600; padding: 3px 10px; border-radius: var(--radius-full); }
.entry-list { max-height: 360px; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; }
.entry-item { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: var(--radius-sm); cursor: pointer; transition: var(--transition); border: 1px solid transparent; }
.entry-item:hover { background: var(--bg-hover); }
.entry-item.selected { background: var(--accent-light); border-color: var(--accent); }
.entry-checkbox input { width: 16px; height: 16px; accent-color: var(--accent); cursor: pointer; }
.entry-index { width: 28px; text-align: center; font-size: 12px; font-weight: 600; color: var(--text-tertiary); flex-shrink: 0; }
.entry-thumb { width: 56px; height: 32px; border-radius: 4px; overflow: hidden; flex-shrink: 0; background: var(--bg-tertiary); }
.entry-thumb img { width: 100%; height: 100%; object-fit: cover; }
.thumb-placeholder { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 16px; }
.entry-info { flex: 1; min-width: 0; }
.entry-title { font-size: 13px; font-weight: 500; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.entry-meta { display: flex; gap: 10px; margin-top: 2px; }
.meta { font-size: 11px; color: var(--text-tertiary); }
.playlist-footer { padding-top: 10px; border-top: 1px solid var(--border-light); margin-top: 8px; }
.footer-text { font-size: 12px; color: var(--text-tertiary); }
</style>