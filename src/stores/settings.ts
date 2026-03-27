import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { AppSettings } from '@/utils/invoke';
import {
  getSettings, saveSettings as apiSaveSettings,
  updateMaxConcurrent, setClipboardWatch,
} from '@/utils/invoke';
import { setLang } from '@/utils/i18n';
import { applyTheme } from '@/utils/theme';
import type { Lang } from '@/utils/i18n';
import type { Theme } from '@/utils/theme';

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    download_path: '', max_concurrent: 3,
    filename_template: '%(title)s.%(ext)s',
    proxy_mode: 'System', proxy_url: null,
    use_browser_cookie: false, browser_type: 'chrome',
    cookie_file_path: null, auto_check_update: true,
    minimize_to_tray: true, clipboard_watch: false,
    theme: 'system', language: 'zh-CN', extra_args: null,
    notify_on_complete: true, notify_on_error: true,
    speed_limit: null, download_thumbnail: false,
    download_metadata: false, audio_quality: '0',
    douyin_api_endpoint: 'https://api.douyin.wtf',
  });
  const isLoading = ref(false);

  async function load() {
    isLoading.value = true;
    try {
      settings.value = await getSettings();
      setLang(settings.value.language as Lang);
      applyTheme(settings.value.theme as Theme);
    } catch (e) {
      console.error('Failed to load settings:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function save() {
    try {
      await apiSaveSettings(settings.value);
      await updateMaxConcurrent(settings.value.max_concurrent);
      await setClipboardWatch(settings.value.clipboard_watch);
      setLang(settings.value.language as Lang);
      applyTheme(settings.value.theme as Theme);
    } catch (e) {
      console.error('Failed to save settings:', e);
      throw e;
    }
  }

  return { settings, isLoading, load, save };
});