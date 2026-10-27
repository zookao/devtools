const KEY = 'devtools:theme'

export type Theme = 'light' | 'dark'

export function getTheme(): Theme {
  return document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light'
}

export function setTheme(t: Theme) {
  document.documentElement.dataset.theme = t
  localStorage.setItem(KEY, t)
}

export function toggleTheme() {
  setTheme(getTheme() === 'dark' ? 'light' : 'dark')
}

/** 启动时初始化：优先用户选择，否则跟随系统 */
export function initTheme() {
  const saved = localStorage.getItem(KEY) as Theme | null
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  setTheme(saved ?? (prefersDark ? 'dark' : 'light'))
}
