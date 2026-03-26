<template>
  <div class="url-input-wrapper">
    <!-- 使用说明 -->
    <p class="input-hint">{{ t('url.hint') }}</p>

    <div class="input-group" :class="{ 'is-loading': store.isFetching, 'is-batch': store.isBatchMode }">
      <span class="input-icon">🔗</span>
      <input
        ref="inputRef"
        v-model="displayValue"
        type="text"
        class="url-input"
        :placeholder="t('url.placeholder')"
        :disabled="store.isFetching || store.isBatchMode"
        :readonly="store.isBatchMode"
        @keydown.enter="handleSubmit"
        @paste="handlePaste"
      />
      <button v-if="displayValue || store.isBatchMode" class="clear-btn" @click="handleClear" :title="t('url.clear')">✕</button>
      <button class="paste-btn btn btn-secondary" @click="handlePasteFromClipboard"
        :disabled="store.isFetching">📋 {{ t('url.paste') }}</button>
      <button
        v-if="!store.isBatchMode"
        class="fetch-btn btn btn-primary"
        @click="handleSubmit"
        :disabled="!displayValue.trim() || store.isFetching"
      >
        <span v-if="store.isFetching" class="spinner"></span><span v-else>🔍</span>
        {{ store.isFetching ? t('url.fetching') : t('url.fetch') }}
      </button>
    </div>

    <!-- 批量检测提示 -->
    <Transition name="fade">
      <div v-if="store.isBatchMode" class="batch-hint">
        <span>📋 {{ t('batch.urlCount', { n: store.batchUrls.length }) }}</span>
        <div class="batch-hint-actions">
          <button class="btn btn-ghost btn-sm" @click="handleClear">✕ {{ t('batch.clearAll') }}</button>
        </div>
      </div>
    </Transition>

    <!-- 错误面板 -->
    <Transition name="fade">
      <div v-if="store.fetchError" class="error-panel">
        <div class="error-header">
          <span>{{ errorIcon }} {{ errorTitle }}</span>
          <button class="btn-ghost close-btn" @click="store.fetchError = null">✕</button>
        </div>
        <div class="error-body">
          <p class="error-msg">{{ friendlyError }}</p>
          <div class="error-suggestions">
             <template v-if="isDouyinError">
              <p class="suggestion-title">🎯 {{ t('error.solutions') }}</p>
              <ol class="douyin-steps">
                <li>{{ t('error.douyinSolution1') }}</li>
                <li>{{ t('error.douyinSolution2') }}</li>
                <li>{{ t('error.douyinSolution3') }}</li>
                <li>{{ t('error.douyinSolution4') }}</li>
              </ol>
              <div class="douyin-alt">
                <p class="suggestion-title">📄 {{ t('error.douyinSolution5') }}</p>
              </div>
            </template>
            <template v-else>
              <p class="suggestion-title">💡 {{ t('error.solutions') }}</p>
              <ul>
                <li v-if="isInvalidUrl"><strong>{{ t('error.solutionValidUrl') }}</strong>，{{ t('error.solutionExample') }} <code>https://www.youtube.com/watch?v=xxxxx</code></li>
                <li v-if="isInvalidUrl">{{ t('error.solutionSupported') }}</li>
                <li v-if="isUnsupportedSite"><strong>{{ t('error.solutionCheckUrl') }}</strong></li>
                <li v-if="isUnsupportedSite">{{ t('error.solutionCheckList') }}</li>
                <li v-if="isCookieError"><strong>{{ t('error.solutionCookie') }}</strong></li>
                <li v-if="isCookieError">{{ t('error.solutionUpdateYtdlp') }} <code>yt-dlp -U</code></li>
                <li v-if="isNetworkError"><strong>{{ t('error.solutionNetwork') }}</strong></li>
                <li v-if="isSslError">{{ t('error.solutionCheckInternet') }}</li>
                <li v-if="isSslError">{{ t('error.solutionSsl') }}</li>
                <li v-if="isSslError">{{ t('error.solutionProxy') }}</li>
                <li v-if="isSslError">{{ t('error.solutionRetryLater') }}</li>
                <li v-if="isVideoUnavailable"><strong>{{ t('error.solutionVideoRemoved') }}</strong></li>
                <li v-if="isGenericError">{{ t('error.solutionCheckComplete') }}</li>
                <li v-if="isGenericError">{{ t('error.solutionTryUpdate') }}</li>
              </ul>
            </template>
          </div>
          <div class="error-actions">
            <button v-if="isDouyinError" class="btn btn-primary btn-sm" @click="openInBrowser">{{ t('error.douyinOpenBrowser') }}</button>
            <button v-if="!isInvalidUrl" class="btn btn-secondary btn-sm" @click="handleRetry">{{ t('error.retry') }}</button>
            <button class="btn btn-secondary btn-sm" @click="copyError">{{ t('error.copyError') }}</button>
            <button v-if="isCookieError || isDouyinError" class="btn btn-primary btn-sm" @click="$emit('go-settings', 'cookie')">{{ t('error.goSettings') }}</button>
            <button v-if="isInvalidUrl" class="btn btn-primary btn-sm" @click="handleClear">{{ t('error.reInput') }}</button>
            
            
            <button v-if="isInvalidUrl" class="btn btn-primary btn-sm" @click="handleClear">{{ t('error.reInput') }}</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useDownloadStore } from '@/stores/download';
import { t } from '@/utils/i18n';
import { isDouyinCookieError } from '@/utils/errors';

const store = useDownloadStore();
const inputRef = ref<HTMLInputElement | null>(null);
const rawUrl = ref(store.currentUrl); // ★ 初始化时从store恢复
defineEmits<{ 'go-settings': [section?: string] }>();

// 输入框显示值：批量模式下显示摘要，否则显示URL
const displayValue = computed({
  get: () => {
    if (store.isBatchMode) {
      return t('batch.urlCount', { n: store.batchUrls.length });
    }
    return rawUrl.value;
  },
  set: (val: string) => {
    if (!store.isBatchMode) {
      rawUrl.value = val;
      store.currentUrl = val; // ★ 同步到store
    }
  },
});

watch(() => store.currentUrl, (val) => {
  if (!store.isBatchMode) rawUrl.value = val;
});

// ===== 工具 =====

function isValidUrl(text: string): boolean {
  return /^https?:\/\/\S+/i.test(text.trim());
}

function extractUrls(text: string): string[] {
  return text
    .split(/[\n\r]+/)
    .map(l => l.trim())
    .filter(l => l.length > 0 && !l.startsWith('#') && !l.startsWith('//'))
    .filter(l => isValidUrl(l));
}

// ===== 核心：处理原始文本 =====

function processRawText(text: string) {
  const trimmed = text.trim();
  if (!trimmed) return;

  const urls = extractUrls(trimmed);

  if (urls.length > 1) {
    // ★ 多个URL → 批量模式
    rawUrl.value = '';
    store.setBatchUrls(urls);
  } else if (urls.length === 1) {
    // ★ 单个URL → 解析
    rawUrl.value = urls[0];
    store.fetchVideo(urls[0]);
  } else {
    // 非URL文本：放入输入框，不自动提交
    rawUrl.value = trimmed.split(/[\n\r]/)[0].trim(); // 只取第一行
  }
}

// ===== 事件 =====

function handlePaste(event: ClipboardEvent) {
  // ★ 关键：拦截paste，从clipboardData读取原始多行文本
  const text = event.clipboardData?.getData('text');
  if (!text) return;

  const urls = extractUrls(text);
  if (urls.length > 1) {
    // 多行URL：阻止默认行为，手动处理
    event.preventDefault();
    store.setBatchUrls(urls);
  } else if (urls.length === 1) {
    // 单URL：阻止默认，手动设置
    event.preventDefault();
    rawUrl.value = urls[0];
    store.fetchVideo(urls[0]);
  }
  // 如果不是URL，让浏览器正常paste到input
}

async function handlePasteFromClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    if (text.trim()) processRawText(text);
  } catch {
    inputRef.value?.focus();
  }
}

function handleSubmit() {
  const input = rawUrl.value.trim();
  if (!input || store.isFetching || store.isBatchMode) return;

  if (!isValidUrl(input)) {
    store.fetchError = t('error.invalidUrlDetail');
    return;
  }
  store.fetchVideo(input);
}

function handleClear() {
  rawUrl.value = '';
  store.clearUrl();
  inputRef.value?.focus();
}

function handleRetry() {
  store.fetchError = null;
  if (rawUrl.value.trim() && !store.isBatchMode) store.fetchVideo(rawUrl.value);
}

async function copyError() {
  try { await navigator.clipboard.writeText(store.fetchError || ''); } catch {}
}

// ===== 错误分类 =====
const isDouyinError = computed(() => {
  const err = store.fetchError || '';
  return isDouyinCookieError(err);
});
const isInvalidUrl = computed(() => {
  const e = store.fetchError || '';
  return e.includes('is not a valid URL') || e.includes('Unsupported URL')
    || /不是有效的/.test(e) || e === t('error.invalidUrlDetail');
});
const isUnsupportedSite = computed(() => {
  const e = store.fetchError || '';
  return !isInvalidUrl.value && (e.includes('Unsupported URL') || e.includes('No video formats found') || e.includes('Unable to extract'));
});
const isCookieError = computed(() => {
  const e = store.fetchError || '';
  return e.includes('cookie') || e.includes('Cookie') || e.includes('Sign in') || e.includes('登录');
});
const isSslError = computed(() => {
  const e = store.fetchError || '';
  return e.includes('SSLError') || e.includes('SSL:') || e.includes('_ssl.c')
    || e.includes('CERTIFICATE_VERIFY_FAILED') || e.includes('ssl.c')
    || e.includes('EOF occurred in violation of protocol');
});
const isNetworkError = computed(() => {
  const e = store.fetchError || '';
  if (isSslError.value) return false;
  return e.includes('timed out') || e.includes('Connection') || e.includes('Network')
    || e.includes('getaddrinfo') || e.includes('URLError')
    || e.includes('Unable to download webpage') || e.includes('No address associated');
});
const isVideoUnavailable = computed(() => {
  const e = store.fetchError || '';
  return e.includes('Video unavailable') || e.includes('Private video') || e.includes('been removed') || e.includes('not available') || e.includes('This video is');
});
const isGenericError = computed(() => !isInvalidUrl.value && !isUnsupportedSite.value && !isCookieError.value && !isNetworkError.value && !isVideoUnavailable.value);

const errorIcon = computed(() => {
  if (isDouyinError.value) return '🎬';
  if (isInvalidUrl.value) return '🔗'; if (isNetworkError.value) return '🌐'; if (isSslError.value) return '🔐';
  if (isCookieError.value) return '🔒'; if (isVideoUnavailable.value) return '🚫'; return '❌';
});
const errorTitle = computed(() => {
  if (isDouyinError.value) return t('error.douyinTitle'); 
  if (isInvalidUrl.value) return t('error.invalidUrl');
  if (isNetworkError.value) return t('error.networkError');
  if (isCookieError.value) return t('error.cookieError');
  if (isVideoUnavailable.value) return t('error.videoUnavailable');
  if (isUnsupportedSite.value) return t('error.unsupportedSite');
  return t('error.fetchFailed');
});
const friendlyError = computed(() => {
  const e = store.fetchError || '';
  if (isDouyinError.value) return t('error.douyinFreshCookie');
  if (e.includes('is not a valid URL')) return t('error.invalidUrlDetail');
  if (e === t('error.invalidUrlDetail')) return e;
  if (e.includes('SSLError') || e.includes('_ssl.c') || e.includes('EOF occurred in violation of protocol')) return t('error.sslDetail');
  if (e.includes('Unsupported URL')) return t('error.unsupportedDetail');
  if (e.includes('timed out') || e.includes('URLError') || e.includes('Unable to download webpage')) return t('error.networkDetail');
  if (e.includes('cookie') || e.includes('Cookie') || e.includes('Sign in')) return t('error.cookieDetail');
  if (e.includes('Video unavailable') || e.includes('Private video')) return t('error.videoUnavailableDetail');
  if (e.includes('not available')) return t('error.regionRestricted');
  if (e.includes('No video formats found')) return t('error.noFormats');
  return e.replace(/^yt-dlp error:\s*/i, '').replace(/^ERROR:\s*(\[.*?\]\s*)?/i, '').trim() || t('error.genericDetail');
});

async function openInBrowser() {
  try {
    const { open } = await import('@tauri-apps/plugin-shell');
    await open(rawUrl.value.trim() || store.currentUrl);
  } catch (e) {
    console.error('Failed to open browser:', e);
  }
}
</script>

<style scoped>
.url-input-wrapper { margin-bottom: 16px; }

.input-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
  padding-left: 2px;
}

.input-group {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-input);
  border: 2px solid var(--border);
  border-radius: var(--radius-md);
  padding: 4px 4px 4px 12px;
  min-height: 48px;
  transition: var(--transition);
}
.input-group:focus-within { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-light); }
.input-group.is-loading { border-color: var(--accent); opacity: 0.8; }
.input-group.is-batch { border-color: var(--accent); background: var(--accent-light); }

.input-icon { font-size: 18px; flex-shrink: 0; }

.url-input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 14px;
  padding: 10px 4px;
  color: var(--text-primary);
  min-width: 0;
}
.url-input:focus { box-shadow: none; border: none; }
.url-input::placeholder { color: var(--text-tertiary); }
.url-input:read-only { color: var(--accent); font-weight: 500; cursor: default; }

.clear-btn {
  width: 24px; height: 24px;
  display: flex; align-items: center; justify-content: center;
  background: var(--bg-tertiary); border: none; border-radius: var(--radius-full);
  color: var(--text-secondary); font-size: 12px; cursor: pointer; flex-shrink: 0;
}
.clear-btn:hover { background: var(--error); color: white; }

.paste-btn, .fetch-btn {
  flex-shrink: 0;
  padding: 8px 14px;
  font-size: 13px;
  border-radius: var(--radius-sm);
}

.batch-hint {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
  padding: 8px 14px;
  background: var(--accent-light);
  color: var(--accent);
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 500;
  animation: slideDown 0.2s ease;
}
.batch-hint-actions { display: flex; gap: 6px; }

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.btn-sm { padding: 4px 10px; font-size: 12px; }

/* 错误面板 */
.error-panel { margin-top: 12px; border: 1px solid var(--error-panel-border); border-radius: var(--radius-md); overflow: hidden; animation: slideDown 0.3s ease; }
.error-header { display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; background: var(--error-panel-header-bg); color: var(--error); font-weight: 600; font-size: 14px; }
.close-btn { font-size: 16px; padding: 2px 6px; color: var(--error); }
.error-body { padding: 12px 14px; background: var(--error-panel-body-bg); }
.error-msg { font-size: 14px; color: var(--text-primary); margin-bottom: 12px; font-weight: 500; }
.error-suggestions { margin-bottom: 12px; }
.suggestion-title { font-size: 13px; font-weight: 600; color: var(--text-secondary); margin-bottom: 6px; }
.error-suggestions ul { list-style: none; padding: 0; }
.error-suggestions li { font-size: 13px; color: var(--text-secondary); padding: 4px 0 4px 16px; position: relative; line-height: 1.5; }
.error-suggestions li::before { content: '•'; position: absolute; left: 4px; color: var(--text-tertiary); }
.error-suggestions code { background: var(--bg-tertiary); padding: 1px 6px; border-radius: 3px; font-size: 12px; font-family: 'Consolas', monospace; }
.error-actions { display: flex; gap: 8px; padding-top: 8px; border-top: 1px solid var(--error-panel-border); }

.spinner { width: 14px; height: 14px; border: 2px solid rgba(255,255,255,0.3); border-top-color: white; border-radius: 50%; animation: spin 0.6s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.douyin-steps {
  padding-left: 22px;
  margin: 8px 0;
}
.douyin-steps li {
  font-size: 13px;
  color: var(--text-secondary);
  padding: 4px 0;
  line-height: 1.6;
}
.douyin-steps li::marker {
  color: var(--accent);
  font-weight: 600;
}
.douyin-alt {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed var(--error-panel-border);
}
</style>