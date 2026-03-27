<template>
  <div class="settings-view">
    <div class="settings-content">
      <section class="settings-section">
        <h3 class="section-title">🔧 {{ t('settings.general') }}</h3>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.savePath') }}</label></div>
          <div class="setting-control path-control">
            <input type="text" :value="settings.download_path" readonly class="path-input" />
            <button class="btn btn-secondary" @click="selectDownloadPath">📁</button>
          </div>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.concurrent') }}</label><span class="setting-desc">{{ t('settings.concurrentDesc') }}</span></div>
          <select v-model.number="settings.max_concurrent" class="setting-select" @change="autoSave">
            <option :value="1">1</option><option :value="2">2</option><option :value="3">3</option><option :value="5">5</option><option :value="8">8</option>
          </select>
        </div>

        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.fileTemplate') }}</label></div>
          <select v-model="settings.filename_template" class="setting-select" @change="autoSave">
            <option value="%(title)s.%(ext)s">{{ t('settings.templateTitle') }}</option>
            <option value="%(title)s [%(id)s].%(ext)s">{{ t('settings.templateTitleId') }}</option>
            <option value="%(playlist_index)s - %(title)s.%(ext)s">{{ t('settings.templateIndexTitle') }}</option>
          </select>
        </div>

        <div class="setting-item">
          <div class="setting-info"><label>🌐 {{ t('settings.language') }}</label></div>
          <select v-model="settings.language" class="setting-select" @change="autoSave">
            <option value="zh-CN">中文</option><option value="en">English</option>
          </select>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>🎨 {{ t('settings.theme') }}</label></div>
          <div class="theme-selector">
            <button v-for="opt in themeOptions" :key="opt.value" class="theme-btn"
              :class="{ active: settings.theme === opt.value }" @click="switchTheme(opt.value)">
              <span class="theme-icon">{{ opt.icon }}</span>
              <span>{{ opt.label }}</span>
            </button>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">⬇️ {{ t('settings.download') }}</h3>
        <div class="setting-item">
          <div class="setting-info">
            <label>{{ t('settings.speedLimit') }}</label>
            <span class="setting-desc">{{ t('settings.speedLimitDesc') }}</span>
          </div>
          <div class="speed-input-group">
            <input
              type="number"
              v-model.number="speedLimitNumber"
              min="0"
              step="1"
              placeholder="0"
              class="setting-input speed-input"
              @blur="handleSpeedChange"
              @keydown.enter="handleSpeedChange"
            />
            <span class="speed-unit">MB/s</span>
          </div>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.audioQuality') }}</label></div>
          <select v-model="settings.audio_quality" class="setting-select" @change="autoSave">
            <option value="0">{{ t('settings.audioQualityBest') }}</option>
            <option value="2">{{ t('settings.audioQualityHigh') }}</option>
            <option value="5">{{ t('settings.audioQualityMedium') }}</option>
          </select>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.downloadThumbnail') }}</label><span class="setting-desc">{{ t('settings.downloadThumbnailDesc') }}</span></div>
          <label class="toggle"><input type="checkbox" v-model="settings.download_thumbnail" @change="autoSave" /><span class="toggle-slider"></span></label>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.downloadMetadata') }}</label><span class="setting-desc">{{ t('settings.downloadMetadataDesc') }}</span></div>
          <label class="toggle"><input type="checkbox" v-model="settings.download_metadata" @change="autoSave" /><span class="toggle-slider"></span></label>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">🔔 {{ t('settings.notification') }}</h3>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.notifyComplete') }}</label><span class="setting-desc">{{ t('settings.notifyCompleteDesc') }}</span></div>
          <label class="toggle"><input type="checkbox" v-model="settings.notify_on_complete" @change="autoSave" /><span class="toggle-slider"></span></label>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.notifyError') }}</label><span class="setting-desc">{{ t('settings.notifyErrorDesc') }}</span></div>
          <label class="toggle"><input type="checkbox" v-model="settings.notify_on_error" @change="autoSave" /><span class="toggle-slider"></span></label>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.minimizeToTray') }}</label><span class="setting-desc">{{ t('settings.minimizeToTrayDesc') }}</span></div>
          <label class="toggle"><input type="checkbox" v-model="settings.minimize_to_tray" @change="autoSave" /><span class="toggle-slider"></span></label>
        </div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.clipboardWatch') }}</label><span class="setting-desc">{{ t('settings.clipboardWatchDesc') }}</span></div>
          <label class="toggle"><input type="checkbox" v-model="settings.clipboard_watch" @change="autoSave" /><span class="toggle-slider"></span></label>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">🌐 {{ t('settings.network') }}</h3>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.proxyMode') }}</label></div>
          <select v-model="settings.proxy_mode" class="setting-select" @change="autoSave">
            <option value="System">{{ t('settings.proxySystem') }}</option>
            <option value="Custom">{{ t('settings.proxyCustom') }}</option>
            <option value="None">{{ t('settings.proxyNone') }}</option>
          </select>
        </div>
        <Transition name="fade">
          <div v-if="settings.proxy_mode === 'Custom'" class="setting-item">
            <div class="setting-info"><label>{{ t('settings.proxyUrl') }}</label></div>
            <input type="text" v-model="settings.proxy_url" placeholder="socks5://127.0.0.1:1080" class="setting-input" @blur="autoSave" />
          </div>
        </Transition>
      </section>

      <section ref="cookieSectionRef" class="settings-section" :class="{ 'section-highlight': isHighlighted }">
        <h3 class="section-title">🍪 {{ t('settings.cookie') }}</h3>
        <p class="section-desc">{{ t('settings.cookieDesc') }}</p>
        <div class="cookie-method">
          <div class="method-header">
            <label class="radio-label">
              <input type="radio" value="file" v-model="cookieMethod" @change="handleCookieMethodChange" />
              <span class="method-title">📄 {{ t('settings.cookieFile') }}</span>
            </label>
          </div>
          <div v-if="cookieMethod === 'file'" class="method-body">
            <p class="method-desc">{{ t('settings.cookieFileDesc') }}</p>

            <!-- 已配置成功 -->
            <div v-if="settings.cookie_file_path" class="cookie-configured">
              <div class="cookie-status success">{{ t('cookie.guideSuccess') }}</div>
              <div class="file-selector">
                <input type="text" :value="settings.cookie_file_path" readonly class="path-input has-file" />
                <button class="btn btn-secondary btn-sm" @click="selectCookieFile">📁 {{ t('settings.select') }}</button>
                <button class="btn btn-secondary btn-sm" @click="clearCookieFile">✕</button>
              </div>
            </div>

            <!-- 未配置：显示引导 -->
            <div v-else class="cookie-guide-wrapper">
              <div class="file-selector" style="margin-bottom: 12px;">
                <input type="text" :value="t('settings.notSelected')" readonly class="path-input" />
                <button class="btn btn-primary" @click="selectCookieFile">📁 {{ t('settings.select') }}</button>
              </div>

              <!-- 展开/收起教程按钮 -->
              <button class="btn btn-secondary guide-toggle-btn" @click="showCookieGuide = !showCookieGuide">
                {{ showCookieGuide ? t('cookie.hideGuide') : t('cookie.showGuide') }}
              </button>

              <!-- 教程内容 -->
              <Transition name="fade">
                <div v-if="showCookieGuide" class="cookie-guide">
                  <h4 class="guide-title">{{ t('cookie.guideTitle') }}</h4>

                  <!-- 步骤 1 -->
                  <div class="guide-step">
                    <div class="step-number">1</div>
                    <div class="step-content">
                      <div class="step-title">{{ t('cookie.guideStep1Title') }}</div>
                      <p class="step-desc">{{ t('cookie.guideStep1Desc') }}</p>
                      <div class="step-note">💡 {{ t('cookie.guideStep1Note') }}</div>
                      <div class="step-actions">
                        <button class="btn btn-primary btn-sm" @click="openExternal('https://chromewebstore.google.com/detail/get-cookiestxt-locally/cclelndahbckbenkjhflpdbgdldlbecc')">
                          {{ t('cookie.guideStep1Btn') }}
                        </button>
                        <span class="store-separator"> | </span>
                        <button class="btn btn-primary btn-sm" @click="openExternal('https://addons.mozilla.org/firefox/addon/get-cookies-txt-locally/')">
                          {{ t('cookie.firefoxStoreBtn') }}
                        </button>
                      </div>
                      <div class="step-alt">
                        <span class="step-alt-text">{{ t('cookie.edgeStore') }}</span>
                        <button class="btn btn-secondary btn-sm" @click="openExternal('https://microsoftedge.microsoft.com/addons/search/get%20cookies.txt%20locally')">
                          {{ t('cookie.edgeStoreBtn') }}
                        </button>
                      </div>
                    </div>
                  </div>

                  <!-- 步骤 2 -->
                  <div class="guide-step">
                    <div class="step-number">2</div>
                    <div class="step-content">
                      <div class="step-title">{{ t('cookie.guideStep2Title') }}</div>
                      <p class="step-desc">{{ t('cookie.guideStep2Desc') }}</p>
                      <p class="step-desc sites-label">{{ t('cookie.guideStep2Sites') }}</p>
                      <div class="quick-sites">
                        <button class="site-btn" @click="openExternal('https://www.douyin.com')">🎵 抖音</button>
                        <button class="site-btn" @click="openExternal('https://www.youtube.com')">▶️ YouTube</button>
                        <button class="site-btn" @click="openExternal('https://www.bilibili.com')">📺 Bilibili</button>
                        <button class="site-btn" @click="openExternal('https://x.com')">🐦 X/Twitter</button>
                      </div>
                    </div>
                  </div>

                  <!-- 步骤 3 -->
                  <div class="guide-step">
                    <div class="step-number">3</div>
                    <div class="step-content">
                      <div class="step-title">{{ t('cookie.guideStep3Title') }}</div>
                      <p class="step-desc">{{ t('cookie.guideStep3Desc') }}</p>
                      <div class="step-note">⚠️ {{ t('cookie.guideStep3Note') }}</div>
                      <div class="export-demo">
                        <div class="demo-box">
                          <div class="demo-header">
                            <span class="demo-icon">🍪</span>
                            <span>Get cookies.txt LOCALLY</span>
                          </div>
                          <div class="demo-body">
                            <div class="demo-site">📄 当前页面: www.douyin.com</div>
                            <button class="demo-export-btn">Export ↓</button>
                          </div>
                            <div class="demo-arrow">{{ t('cookie.guideStep3Demo') }}</div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 步骤 4 -->
                  <div class="guide-step">
                    <div class="step-number">4</div>
                    <div class="step-content">
                      <div class="step-title">{{ t('cookie.guideStep4Title') }}</div>
                      <p class="step-desc">{{ t('cookie.guideStep4Desc') }}</p>
                      <button class="btn btn-primary" @click="selectCookieFile">
                        {{ t('cookie.guideStep4Btn') }}
                      </button>
                    </div>
                  </div>

                  <!-- 使用提示 -->
                  <div class="guide-tips">
                    <div class="tips-title">💡 {{ t('cookie.guideTips') }}</div>
                    <ul>
                      <li>{{ t('cookie.guideTip1') }}</li>
                      <li>{{ t('cookie.guideTip2') }}</li>
                      <li>{{ t('cookie.guideTip3') }}</li>
                    </ul>
                  </div>
                </div>
              </Transition>
            </div>
          </div>
        </div>

        <div class="cookie-method">
          <div class="method-header">
            <label class="radio-label">
              <input type="radio" value="browser" v-model="cookieMethod" @change="handleCookieMethodChange" />
              <span class="method-title">🌐 {{ t('settings.cookieBrowser') }}</span>
              <span class="method-warn">⚠️ {{ t('settings.cookieBrowserWarn') }}</span>
            </label>
          </div>
          <div v-if="cookieMethod === 'browser'" class="method-body">
            <div class="setting-item">
              <div class="setting-info"><label>{{ t('settings.browser') }}</label></div>
              <select v-model="settings.browser_type" class="setting-select" @change="autoSave">
                <option value="chrome">Chrome</option><option value="edge">Edge</option><option value="firefox">Firefox</option>
              </select>
            </div>

            <!-- ★ Cookie 诊断按钮 -->
            <div class="cookie-diagnose-bar">
              <button class="btn btn-secondary" @click="runDiagnose" :disabled="isDiagnosing">
                <span v-if="isDiagnosing" class="spinner-sm"></span>
                {{ isDiagnosing ? '诊断中...' : '🔍 诊断 Cookie 读取' }}
              </button>
              <span class="diagnose-hint">检查浏览器 Cookie 是否可以正常读取</span>
            </div>

            <!-- ★ 诊断结果面板 -->
            <Transition name="fade">
              <div v-if="diagResult" class="diagnose-panel">
                <div class="diagnose-header">
                  <span class="diagnose-title">
                    <span v-if="diagResult.overall === 'ok'">✅ Cookie 可用</span>
                    <span v-else-if="diagResult.overall === 'warn'">⚠️ Cookie 可能有问题</span>
                    <span v-else>❌ Cookie 读取失败</span>
                  </span>
                  <button class="btn-ghost close-btn" @click="diagResult = null">✕</button>
                </div>
                <div class="diagnose-body">
                  <!-- 检查项列表 -->
                  <div v-for="(check, i) in diagResult.checks" :key="i" class="diag-row">
                    <span class="diag-icon">{{ check.icon }}</span>
                    <span class="diag-label">{{ check.label }}</span>
                    <span class="diag-value" :class="check.cls">{{ check.value }}</span>
                  </div>

                  <!-- Cookie 数据库路径 -->
                  <div v-if="diagResult.db_paths && diagResult.db_paths.length > 0" class="diag-section-block">
                    <div class="diag-section-title">Cookie 数据库文件</div>
                    <div v-for="db in diagResult.db_paths" :key="db.path" class="db-row">
                      <span :class="db.exists ? 'diag-ok' : 'diag-err'">{{ db.exists ? '✅' : '❌' }}</span>
                      <span class="db-path" :title="db.path">{{ shortenPath(db.path) }}</span>
                      <span v-if="db.exists" class="db-size">{{ formatBytes(db.size) }}</span>
                    </div>
                  </div>

                  <!-- yt-dlp 实际测试结果 -->
                  <div v-if="diagResult.test_output" class="diag-section-block">
                    <div class="diag-section-title">yt-dlp 测试输出</div>
                    <pre class="diag-output">{{ diagResult.test_output }}</pre>
                  </div>

                  <!-- 建议 -->
                  <div v-if="diagResult.suggestions.length > 0" class="diag-section-block">
                    <div class="diag-section-title">💡 建议</div>
                    <ul class="diag-suggestions">
                      <li v-for="(s, i) in diagResult.suggestions" :key="i">{{ s }}</li>
                    </ul>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>

        <div class="cookie-method">
          <div class="method-header">
            <label class="radio-label">
              <input type="radio" value="none" v-model="cookieMethod" @change="handleCookieMethodChange" />
              <span class="method-title">🚫 {{ t('settings.cookieNone') }}</span>
            </label>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">⚡ {{ t('settings.advanced') }}</h3>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.ytdlpVersion') }}</label></div>
          <div class="version-display">
            <span class="version-text">{{ ytdlpVersion || t('settings.checking') }}</span>
            <!--button class="btn btn-secondary btn-sm" @click="checkVersion" :disabled="isUpdating">🔄</button-->
            <button class="btn btn-primary btn-sm" @click="handleUpdateYtdlp" :disabled="isUpdating">
              {{ isUpdating ? t('settings.updating') : t('settings.ytdlpUpdate') }}
            </button>
          </div>
        </div>
        <div v-if="updateResult" class="update-result" :class="updateResultType">{{ updateResult }}</div>
        <div class="setting-item">
          <div class="setting-info"><label>{{ t('settings.extraArgs') }}</label><span class="setting-desc">{{ t('settings.extraArgsDesc') }}</span></div>
          <input type="text" v-model="settings.extra_args" placeholder="--limit-rate 10M" class="setting-input" @blur="autoSave" />
        </div>
      </section>

      <div class="settings-actions">
        <button class="btn btn-primary" @click="handleSave" :disabled="isSaving">
          {{ isSaving ? t('settings.saving') : '💾 ' + t('settings.save') }}
        </button>
        <button class="btn btn-secondary" @click="handleReset">↩️ {{ t('settings.reset') }}</button>
        <span v-if="lastSaveTime" class="save-hint">✅ {{ lastSaveTime }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useSettingsStore } from '@/stores/settings';
import {
  checkYtdlp, updateYtdlp,
  diagnoseCookie,
} from '@/utils/invoke';
import type { CookieDiagnostic } from '@/utils/invoke';
import { t } from '@/utils/i18n';

interface DiagCheck {
  icon: string;
  label: string;
  value: string;
  cls: string;
}

interface DiagDbPath {
  path: string;
  exists: boolean;
  size: number;
}

interface DiagResultUI {
  overall: 'ok' | 'warn' | 'error';
  checks: DiagCheck[];
  db_paths: DiagDbPath[];
  test_output: string;
  suggestions: string[];
}

const props = defineProps<{ scrollTo?: string | null }>();

const settingsStore = useSettingsStore();
const settings = computed(() => settingsStore.settings);
const isSaving = ref(false);
const ytdlpVersion = ref('');
const lastSaveTime = ref('');
const cookieMethod = ref<'none' | 'file' | 'browser'>('none');
const isUpdating = ref(false);
const updateResult = ref('');
const updateResultType = ref('');
const cookieSectionRef = ref<HTMLElement | null>(null);
const isHighlighted = ref(false);
const speedLimitNumber = ref<number | null>(null);

// Cookie 诊断
const isDiagnosing = ref(false);
const diagResult = ref<DiagResultUI | null>(null);

const themeOptions = computed(() => [
  { value: 'light', icon: '☀️', label: t('settings.themeLight') },
  { value: 'dark', icon: '🌙', label: t('settings.themeDark') },
  { value: 'system', icon: '💻', label: t('settings.themeSystem') },
]);
const showCookieGuide = ref(false);

// 添加打开外部链接方法：
async function openExternal(url: string) {
  try {
    const { open } = await import('@tauri-apps/plugin-shell');
    await open(url);
  } catch (e) {
    console.error('Failed to open URL:', e);
    // 降级：用 window.open
    window.open(url, '_blank');
  }
}

function switchTheme(theme: string) {
  settings.value.theme = theme;
  autoSave();
}

function scrollToSection(section: string) {
  if (section === 'cookie' && cookieSectionRef.value) {
    nextTick(() => {
      cookieSectionRef.value?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      isHighlighted.value = true;
      setTimeout(() => { isHighlighted.value = false; }, 2000);
    });
  }
}

function handleSpeedChange() {
  if (speedLimitNumber.value && speedLimitNumber.value > 0) {
    settings.value.speed_limit = `${speedLimitNumber.value}M`;
  } else {
    settings.value.speed_limit = null;
    speedLimitNumber.value = null;
  }
  autoSave();
}

function initSpeedLimit() {
  const raw = settings.value.speed_limit;
  if (!raw) { speedLimitNumber.value = null; return; }
  const match = raw.match(/^(\d+\.?\d*)/);
  speedLimitNumber.value = match ? parseFloat(match[1]) : null;
}

// ===== Cookie 诊断 =====
async function runDiagnose() {
  isDiagnosing.value = true;
  diagResult.value = null;
  try {
    // 先保存当前设置
    await settingsStore.save();
    const raw: CookieDiagnostic = await diagnoseCookie();
    diagResult.value = transformDiagResult(raw);
  } catch (e: any) {
    diagResult.value = {
      overall: 'error',
      checks: [{ icon: '❌', label: '诊断执行', value: String(e.message || e), cls: 'diag-err' }],
      db_paths: [],
      test_output: '',
      suggestions: ['请检查 yt-dlp 是否正确安装'],
    };
  } finally {
    isDiagnosing.value = false;
  }
}

function transformDiagResult(raw: CookieDiagnostic): DiagResultUI {
  const checks: DiagCheck[] = [];
  let overall: 'ok' | 'warn' | 'error' = 'ok';

  // 1. 配置方式
  checks.push({
    icon: '⚙️',
    label: '配置方式',
    value: raw.method === 'browser' ? `从 ${raw.browser_type} 浏览器读取` : raw.method === 'file' ? 'Cookie 文件' : '未配置',
    cls: '',
  });

  // 2. 浏览器运行状态
  if (raw.method === 'browser') {
    if (raw.browser_running) {
      checks.push({ icon: '⚠️', label: '浏览器状态', value: `${raw.browser_type} 正在运行（可能导致 Cookie 锁定）`, cls: 'diag-warn' });
      overall = 'warn';
    } else {
      checks.push({ icon: '✅', label: '浏览器状态', value: `${raw.browser_type} 未运行`, cls: 'diag-ok' });
    }
  }

  // 3. Cookie 数据库
  if (raw.method === 'browser') {
    if (raw.cookie_db_found) {
      checks.push({ icon: '✅', label: 'Cookie 数据库', value: '找到有效的数据库文件', cls: 'diag-ok' });
    } else {
      checks.push({ icon: '❌', label: 'Cookie 数据库', value: '未找到数据库文件', cls: 'diag-err' });
      overall = 'error';
    }
  }

  // 4. Cookie 文件
  if (raw.method === 'file') {
    if (raw.cookie_file_exists && raw.cookie_file_size > 0) {
      checks.push({ icon: '✅', label: 'Cookie 文件', value: `存在 (${formatBytes(raw.cookie_file_size)})`, cls: 'diag-ok' });
    } else if (raw.cookie_file_exists) {
      checks.push({ icon: '⚠️', label: 'Cookie 文件', value: '文件为空', cls: 'diag-warn' });
      overall = 'warn';
    } else {
      checks.push({ icon: '❌', label: 'Cookie 文件', value: '文件不存在', cls: 'diag-err' });
      overall = 'error';
    }
  }

  // 5. 实际测试结果
  if (raw.test_result) {
    const isOk = raw.test_result.includes('成功') || raw.test_result.includes('success');
    checks.push({
      icon: isOk ? '✅' : '❌',
      label: 'yt-dlp 实测',
      value: isOk ? 'Cookie 读取成功' : 'Cookie 读取失败',
      cls: isOk ? 'diag-ok' : 'diag-err',
    });
    if (!isOk) overall = 'error';
  }

  return {
    overall,
    checks,
    db_paths: raw.cookie_db_paths || [],
    test_output: raw.test_result || '',
    suggestions: raw.suggestions || [],
  };
}

function shortenPath(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/');
  if (parts.length <= 3) return path;
  return '.../' + parts.slice(-3).join('/');
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB'];
  let s = bytes, i = 0;
  while (s >= 1024 && i < units.length - 1) { s /= 1024; i++; }
  return `${s.toFixed(1)} ${units[i]}`;
}

onMounted(async () => {
  await checkVersion();
  if (settings.value.cookie_file_path) cookieMethod.value = 'file';
  else if (settings.value.use_browser_cookie) cookieMethod.value = 'browser';
  else cookieMethod.value = 'none';
  initSpeedLimit();
  if (props.scrollTo) scrollToSection(props.scrollTo);
});

watch(() => props.scrollTo, (val) => { if (val) scrollToSection(val); });
watch(() => settings.value.speed_limit, () => { initSpeedLimit(); });

async function checkVersion() {
  try { ytdlpVersion.value = await checkYtdlp(); }
  catch { ytdlpVersion.value = t('settings.notFound'); }
}

async function handleUpdateYtdlp() {
  isUpdating.value = true; updateResult.value = '';
  try {
    const r = await updateYtdlp();
    updateResult.value = r; updateResultType.value = 'success';
    await checkVersion();
  } catch (e: any) {
    updateResult.value = typeof e === 'string' ? e : e.message || 'Update failed';
    updateResultType.value = 'error';
  } finally {
    isUpdating.value = false;
    setTimeout(() => { updateResult.value = ''; }, 5000);
  }
}

function handleCookieMethodChange() {
  diagResult.value = null; // 切换方式时清除诊断结果
  if (cookieMethod.value === 'file') {
    settings.value.use_browser_cookie = false;
  } else if (cookieMethod.value === 'browser') {
    settings.value.use_browser_cookie = true;
    settings.value.cookie_file_path = null;
    autoSave();
  } else {
    settings.value.use_browser_cookie = false;
    settings.value.cookie_file_path = null;
    autoSave();
  }
}

async function selectCookieFile() {
  const s = await openDialog({
    title: 'cookies.txt',
    filters: [{ name: 'Cookie', extensions: ['txt'] }, { name: 'All', extensions: ['*'] }],
  });
  if (s) {
    settings.value.cookie_file_path = s as string;
    settings.value.use_browser_cookie = false;
    await autoSave();
  }
}

function clearCookieFile() { settings.value.cookie_file_path = null; autoSave(); }

async function selectDownloadPath() {
  const s = await openDialog({ directory: true, title: t('options.selectDir') });
  if (s) { settings.value.download_path = s as string; await autoSave(); }
}

async function autoSave() {
  try { await settingsStore.save(); lastSaveTime.value = new Date().toLocaleTimeString('zh-CN'); }
  catch (e) { console.error(e); }
}

async function handleSave() {
  isSaving.value = true;
  try { await settingsStore.save(); lastSaveTime.value = new Date().toLocaleTimeString('zh-CN'); }
  finally { isSaving.value = false; }
}

function handleReset() {
  if (!confirm(t('settings.resetConfirm'))) return;
  settingsStore.settings = {
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
  };
  cookieMethod.value = 'none';
  diagResult.value = null;
  autoSave();
}
</script>

<style scoped>
.settings-view { max-width: 640px; margin: 0 auto; }
.settings-section { margin-bottom: 28px; padding: 20px; border-bottom: 1px solid var(--border-light); border-radius: var(--radius-md); border: 1px solid transparent; transition: border-color 0.3s ease, box-shadow 0.3s ease; }
.section-highlight { border-color: var(--accent) !important; box-shadow: 0 0 0 3px var(--accent-light), var(--shadow-md); animation: highlightPulse 2s ease-in-out; }
@keyframes highlightPulse { 0% { border-color: transparent; box-shadow: none; } 15% { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light), var(--shadow-md); } 85% { border-color: var(--accent); } 100% { border-color: transparent; box-shadow: none; } }
.section-title { font-size: 16px; font-weight: 600; margin-bottom: 4px; color: var(--text-primary); }
.section-desc { font-size: 13px; color: var(--text-tertiary); margin-bottom: 16px; }
.setting-item { display: flex; justify-content: space-between; align-items: center; padding: 12px 0; gap: 16px; }
.setting-info { flex: 1; min-width: 0; }
.setting-info label { font-size: 14px; font-weight: 500; color: var(--text-primary); display: block; }
.setting-desc { font-size: 12px; color: var(--text-tertiary); margin-top: 2px; display: block; }
.setting-select, .setting-input { min-width: 200px; padding: 7px 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-primary); font-size: 13px; }
.path-control { display: flex; gap: 8px; flex: 1; max-width: 360px; }
.path-control .path-input, .file-selector .path-input { flex: 1; padding: 7px 12px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-tertiary); font-size: 13px; }
.path-input.has-file { color: var(--text-primary); }
.version-display { display: flex; align-items: center; gap: 8px; }
.version-text { font-size: 13px; color: var(--text-secondary); font-family: monospace; }
.btn-sm { padding: 4px 10px; font-size: 12px; }
.theme-selector { display: flex; gap: 6px; }
.theme-btn { display: flex; align-items: center; gap: 4px; padding: 6px 14px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--bg-input); color: var(--text-secondary); font-size: 13px; cursor: pointer; transition: var(--transition); }
.theme-btn:hover { border-color: var(--accent); color: var(--text-primary); }
.theme-btn.active { border-color: var(--accent); background: var(--accent-light); color: var(--accent); font-weight: 600; }
.theme-icon { font-size: 14px; }
.speed-input-group { display: flex; align-items: center; gap: 0; border: 1px solid var(--border); border-radius: var(--radius-sm); overflow: hidden; background: var(--bg-input); }
.speed-input { width: 80px; min-width: 80px; border: none !important; border-radius: 0 !important; text-align: right; padding-right: 4px; background: transparent !important; -moz-appearance: textfield; }
.speed-input::-webkit-inner-spin-button, .speed-input::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
.speed-input:focus { box-shadow: none !important; }
.speed-input-group:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }
.speed-unit { padding: 7px 10px 7px 4px; font-size: 13px; color: var(--text-tertiary); font-weight: 500; white-space: nowrap; user-select: none; }
.toggle { position: relative; display: inline-block; width: 44px; height: 24px; flex-shrink: 0; }
.toggle input { opacity: 0; width: 0; height: 0; }
.toggle-slider { position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background: var(--bg-tertiary); border-radius: 12px; transition: 0.3s; }
.toggle-slider::before { content: ''; position: absolute; height: 18px; width: 18px; left: 3px; bottom: 3px; background: white; border-radius: 50%; transition: 0.3s; }
.toggle input:checked + .toggle-slider { background: var(--accent); }
.toggle input:checked + .toggle-slider::before { transform: translateX(20px); }
.cookie-method { border: 1px solid var(--border-light); border-radius: var(--radius-md); margin-bottom: 10px; overflow: hidden; }
.method-header { padding: 12px 14px; background: var(--bg-secondary); }
.radio-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 14px; }
.radio-label input[type="radio"] { width: 16px; height: 16px; accent-color: var(--accent); }
.method-title { font-weight: 600; color: var(--text-primary); }
.method-warn { font-size: 12px; color: var(--warning); font-weight: normal; }
.method-body { padding: 14px; border-top: 1px solid var(--border-light); }
.method-desc { font-size: 13px; color: var(--text-secondary); margin-bottom: 12px; }
.file-selector { display: flex; gap: 8px; align-items: center; }
.cookie-status { margin-top: 10px; padding: 8px 12px; border-radius: var(--radius-sm); font-size: 13px; }
.cookie-status.success { background: var(--cookie-success-bg); color: var(--cookie-success-text); }
.update-result { padding: 8px 14px; border-radius: var(--radius-sm); font-size: 13px; margin: 8px 0; }
.update-result.success { background: var(--success-msg-bg); color: var(--success-msg-text); }
.update-result.error { background: var(--error-msg-bg); color: var(--error-msg-text); }
.settings-actions { display: flex; align-items: center; gap: 12px; padding-top: 8px; }
.save-hint { font-size: 12px; color: var(--success); }

/* ===== Cookie 诊断样式 ===== */
.cookie-diagnose-bar {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
  display: flex;
  align-items: center;
  gap: 12px;
}
.diagnose-hint { font-size: 12px; color: var(--text-tertiary); }

.spinner-sm {
  display: inline-block;
  width: 14px; height: 14px;
  border: 2px solid var(--text-tertiary);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  margin-right: 4px;
}
@keyframes spin { to { transform: rotate(360deg); } }

.diagnose-panel {
  margin-top: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow: hidden;
  animation: slideDown 0.3s ease;
}
@keyframes slideDown {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}

.diagnose-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
}
.diagnose-title { font-size: 14px; font-weight: 600; }
.close-btn { font-size: 16px; padding: 2px 6px; }

.diagnose-body { padding: 14px; }

.diag-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid var(--border-light);
}
.diag-row:last-child { border-bottom: none; }
.diag-icon { font-size: 14px; flex-shrink: 0; width: 20px; text-align: center; }
.diag-label { font-size: 13px; color: var(--text-secondary); min-width: 100px; flex-shrink: 0; }
.diag-value { font-size: 13px; color: var(--text-primary); flex: 1; }
.diag-ok { color: var(--success) !important; }
.diag-warn { color: var(--warning) !important; }
.diag-err { color: var(--error) !important; }

.diag-section-block {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
}
.diag-section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.db-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 0;
  font-size: 12px;
}
.db-path {
  color: var(--text-secondary);
  font-family: 'Consolas', 'Courier New', monospace;
  word-break: break-all;
  flex: 1;
}
.db-size { color: var(--text-tertiary); font-size: 11px; flex-shrink: 0; }

.diag-output {
  background: var(--bg-input);
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
  max-height: 150px;
  overflow-y: auto;
  color: var(--text-secondary);
  line-height: 1.5;
}

.diag-suggestions {
  list-style: none;
  padding: 0;
  margin: 0;
}
.diag-suggestions li {
  padding: 4px 0 4px 18px;
  position: relative;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
}
.diag-suggestions li::before {
  content: '💡';
  position: absolute;
  left: 0;
  font-size: 12px;
}
/* ===== Cookie 引导教程样式 ===== */
.cookie-configured { display: flex; flex-direction: column; gap: 10px; }

.cookie-guide-wrapper { display: flex; flex-direction: column; gap: 10px; }

.guide-toggle-btn {
  align-self: flex-start;
  font-size: 13px;
}

.cookie-guide {
  margin-top: 8px;
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  animation: slideDown 0.3s ease;
}

.guide-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 16px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-light);
}

.guide-step {
  display: flex;
  gap: 14px;
  padding: 14px 0;
  border-bottom: 1px solid var(--border-light);
}
.guide-step:last-of-type { border-bottom: none; }

.step-number {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--accent);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 700;
  flex-shrink: 0;
}

.step-content { flex: 1; min-width: 0; }

.step-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.step-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin-bottom: 8px;
}

.step-note {
  font-size: 12px;
  color: var(--warning);
  background: var(--warn-msg-bg);
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  margin-bottom: 10px;
}

.step-actions { margin-bottom: 8px; }

.step-alt {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--border-light);
}
.step-alt-text {
  font-size: 12px;
  color: var(--text-tertiary);
}

.sites-label { margin-bottom: 4px !important; }

.quick-sites {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.site-btn {
  padding: 4px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: var(--transition);
}
.site-btn:hover {
  border-color: var(--accent);
  background: var(--accent-light);
  color: var(--accent);
}

/* 扩展操作演示 */
.export-demo {
  margin-top: 10px;
  display: flex;
  justify-content: center;
}

.demo-box {
  width: 240px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--bg-card);
  box-shadow: var(--shadow-sm);
}

.demo-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}
.demo-icon { font-size: 16px; }

.demo-body {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
}
.demo-site {
  font-size: 11px;
  color: var(--text-tertiary);
}

.demo-export-btn {
  padding: 6px 24px;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 600;
  cursor: default;
  animation: pulse 2s ease-in-out infinite;
}

.demo-arrow {
  text-align: center;
  padding: 6px;
  font-size: 12px;
  color: var(--accent);
  font-weight: 600;
  background: var(--accent-light);
}

@keyframes pulse {
  0%, 100% { transform: scale(1); box-shadow: none; }
  50% { transform: scale(1.05); box-shadow: 0 0 12px rgba(76, 110, 245, 0.4); }
}

.guide-tips {
  margin-top: 14px;
  padding: 12px;
  background: var(--bg-input);
  border-radius: var(--radius-sm);
}
.tips-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}
.guide-tips ul {
  list-style: none;
  padding: 0;
}
.guide-tips li {
  font-size: 12px;
  color: var(--text-tertiary);
  padding: 3px 0 3px 16px;
  position: relative;
  line-height: 1.5;
}
.guide-tips li::before {
  content: '•';
  position: absolute;
  left: 4px;
  color: var(--text-tertiary);
}
</style>