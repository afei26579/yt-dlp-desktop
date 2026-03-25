import { ref, computed } from 'vue';

export type Theme = 'light' | 'dark' | 'system';

const currentTheme = ref<Theme>('system');
const effectiveTheme = ref<'light' | 'dark'>('light');

function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function applyTheme(theme: Theme) {
  currentTheme.value = theme;

  if (theme === 'system') {
    effectiveTheme.value = getSystemTheme();
  } else {
    effectiveTheme.value = theme;
  }

  document.documentElement.setAttribute('data-theme', effectiveTheme.value);

  // 同步到 localStorage 用于防止闪烁
  try {
    localStorage.setItem('app-theme', theme);
  } catch {}
}

export function initThemeListener() {
  if (typeof window === 'undefined') return;
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (currentTheme.value === 'system') {
      effectiveTheme.value = getSystemTheme();
      document.documentElement.setAttribute('data-theme', effectiveTheme.value);
    }
  });
}

export function useTheme() {
  return {
    currentTheme: computed(() => currentTheme.value),
    effectiveTheme: computed(() => effectiveTheme.value),
    applyTheme,
  };
}