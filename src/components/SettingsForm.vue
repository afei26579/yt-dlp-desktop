<template>
  <div class="settings-form">
    <!-- 通用设置 -->
    <section class="settings-section">
      <h3 class="section-title">🔧 通用设置</h3>

      <div class="setting-item">
        <div class="setting-info">
          <label>默认保存路径</label>
        </div>
        <div class="setting-control path-control">
          <input
            type="text"
            :value="settings.download_path"
            readonly
            class="path-input"
          />
          <button class="btn btn-secondary" @click="selectDownloadPath">📁</button>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label>同时下载数量</label>
        </div>
        <select v-model.number="settings.max_concurrent" class="setting-select">
          <option :value="1">1</option>
          <option :value="2">2</option>
          <option :value="3">3</option>
          <option :value="5">5</option>
        </select>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label>文件命名模板</label>
        </div>
        <select v-model="settings.filename_template" class="setting-select">
          <option value="%(title)s.%(ext)s">标题.扩展名</option>
          <option value="%(title)s [%(id)s].%(ext)s">标题 [ID].扩展名</option>
          <option value="%(uploader)s - %(title)s.%(ext)s">上传者 - 标题.扩展名</option>
          <option value="%(upload_date)s - %(title)s.%(ext)s">日期 - 标题.扩展名</option>
        </select>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label>最小化到托盘</label>
          <span class="setting-desc">关闭窗口时最小化到系统托盘</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="settings.minimize_to_tray" />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </section>

    <!-- 网络设置 -->
    <section class="settings-section">
      <h3 class="section-title">🌐 网络设置</h3>

      <div class="setting-item">
        <div class="setting-info">
          <label>代理模式</label>
        </div>
        <select v-model="settings.proxy_mode" class="setting-select">
          <option value="None">无代理</option>
          <option value="System">系统代理</option>
          <option value="Custom">自定义</option>
        </select>
      </div>

      <Transition name="fade">
        <div v-if="settings.proxy_mode === 'Custom'" class="setting-item">
          <div class="setting-info">
            <label>代理地址</label>
          </div>
          <input
            type="text"
            v-model="settings.proxy_url"
            placeholder="socks5://127.0.0.1:1080"
            class="setting-input"
          />
        </div>
      </Transition>
    </section>

    <!-- Cookie 设置 -->
    <section class="settings-section">
      <h3 class="section-title">🍪 Cookie 设置</h3>
      <p class="section-desc">用于下载需要登录的视频</p>

      <div class="setting-item">
        <div class="setting-info">
          <label>使用浏览器 Cookie</label>
          <span class="setting-desc">自动从浏览器读取登录状态</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="settings.use_browser_cookie" />
          <span class="toggle-slider"></span>
        </label>
      </div>

      <Transition name="fade">
        <div v-if="settings.use_browser_cookie" class="setting-item">
          <div class="setting-info">
            <label>浏览器</label>
          </div>
          <select v-model="settings.browser_type" class="setting-select">
            <option value="chrome">Chrome</option>
            <option value="firefox">Firefox</option>
            <option value="edge">Edge</option>
            <option value="brave">Brave</option>
          </select>
        </div>
      </Transition>
    </section>

    <!-- 高级设置 -->
    <section class="settings-section">
      <h3 class="section-title">⚡ 高级设置</h3>

      <div class="setting-item">
        <div class="setting-info">
          <label>抖音 API 端点</label>
          <span class="setting-desc">用于下载抖音视频（留空则仅使用 yt-dlp）</span>
        </div>
        <input
          type="text"
          v-model="settings.douyin_api_endpoint"
          placeholder="https://api.douyin.wtf"
          class="setting-input"
        />
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label>yt-dlp 版本</label>
        </div>
        <div class="version-display">
          <span class="version-text">{{ ytdlpVersion || '检测中...' }}</span>
          <button class="btn btn-secondary btn-sm" @click="$emit('check-version')">
            🔄 检查
          </button>
        </div>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <label>额外命令行参数</label>
          <span class="setting-desc">追加到每次下载命令中</span>
        </div>
        <input
          type="text"
          v-model="settings.extra_args"
          placeholder="例如: --limit-rate 10M"
          class="setting-input"
        />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';
import type { AppSettings } from '@/utils/invoke';

const props = defineProps<{
  settings: AppSettings;
  ytdlpVersion: string;
}>();

defineEmits<{
  'check-version': [];
}>();

async function selectDownloadPath() {
  const selected = await open({
    directory: true,
    defaultPath: props.settings.download_path,
    title: '选择下载目录',
  });
  if (selected) {
    props.settings.download_path = selected as string;
  }
}
</script>

<style scoped>
.settings-section {
  margin-bottom: 28px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border-light);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
  color: var(--text-primary);
}

.section-desc {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-bottom: 16px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  gap: 16px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-info label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  display: block;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
  display: block;
}

.setting-select,
.setting-input {
  min-width: 200px;
  padding: 7px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: 13px;
}

.path-control {
  display: flex;
  gap: 8px;
  flex: 1;
  max-width: 360px;
}

.path-control .path-input {
  flex: 1;
  padding: 7px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: default;
}

.version-display {
  display: flex;
  align-items: center;
  gap: 10px;
}

.version-text {
  font-size: 13px;
  color: var(--text-secondary);
  font-family: monospace;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

/* Toggle Switch */
.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-tertiary);
  border-radius: 24px;
  transition: 0.3s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  border-radius: 50%;
  transition: 0.3s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.toggle input:checked + .toggle-slider {
  background-color: var(--accent);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(20px);
}
</style>