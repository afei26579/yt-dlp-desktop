import { t } from './i18n';

/**
 * 将 yt-dlp 原始错误消息转换为用户友好的提示
 */
export function friendlyDownloadError(raw: string): string {
  if (!raw) return t('error.genericDetail');

  const e = raw.trim();

   // ★ 抖音 Fresh Cookie 专属错误
  if (e.startsWith('DOUYIN_FRESH_COOKIE|') || e.includes('Fresh cookies')) {
    return t('error.douyinFreshCookie');
  }
  // SSL 错误
  if (e.includes('SSLError') || e.includes('_ssl.c') || e.includes('EOF occurred in violation of protocol')) {
    return t('error.sslDetail');
  }

  // 网络超时/断连
  if (e.includes('timed out') || e.includes('URLError') || e.includes('Unable to download')
    || e.includes('Connection') && (e.includes('reset') || e.includes('refused') || e.includes('aborted'))
    || e.includes('getaddrinfo') || e.includes('No address associated')
    || e.includes('Network is unreachable') || e.includes('Temporary failure in name resolution')) {
    return t('error.networkDetail');
  }

  // HTTP 错误
  if (e.includes('HTTP Error 403')) return t('error.http403');
  if (e.includes('HTTP Error 404')) return t('error.http404');
  if (e.includes('HTTP Error 429')) return t('error.http429');
  if (/HTTP Error [45]\d\d/.test(e)) return t('error.httpGeneric');

  // Cookie/登录
  if (e.includes('cookie') || e.includes('Cookie') || e.includes('Sign in') || e.includes('登录')) {
    return t('error.cookieDetail');
  }

  // 视频不可用
  if (e.includes('Video unavailable') || e.includes('Private video') || e.includes('been removed')) {
    return t('error.videoUnavailableDetail');
  }

  // 地区限制
  if (e.includes('not available') || e.includes('blocked') || e.includes('geo')) {
    return t('error.regionRestricted');
  }

  // 无格式
  if (e.includes('No video formats') || e.includes('Requested format')) {
    return t('error.noFormats');
  }

  // 磁盘空间
  if (e.includes('No space left') || e.includes('disk full')) {
    return t('error.diskFull');
  }

  // 权限
  if (e.includes('Permission denied') || e.includes('Access is denied')) {
    return t('error.permissionDenied');
  }

  // 清理原始错误前缀
  return e
    .replace(/^DOUYIN_FRESH_COOKIE\|/i, '')
    .replace(/^ERROR:\s*(\[.*?\]\s*)*/gi, '')
    .replace(/^Got error:\s*/i, '')
    .replace(/\.\s*Giving up after \d+ retries\.?/i, '')
    .trim() || t('error.genericDetail');
}

/**
 * 判断错误是否为网络相关（可重试）
 */
export function isRetryableError(raw: string): boolean {
  if (!raw) return false;
  const e = raw.toLowerCase();
  return e.includes('ssl') || e.includes('timed out') || e.includes('connection')
    || e.includes('network') || e.includes('urlerror') || e.includes('getaddrinfo')
    || e.includes('eof occurred') || e.includes('http error 429')
    || e.includes('http error 503') || e.includes('temporary');
}

// ★ 新增：判断是否为抖音 cookie 错误
export function isDouyinCookieError(raw: string): boolean {
  if (!raw) return false;
  return raw.includes('DOUYIN_FRESH_COOKIE') || raw.includes('Fresh cookies');
}