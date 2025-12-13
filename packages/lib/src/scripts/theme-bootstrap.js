/**
 * Theme Bootstrap Script
 * 
 * This script MUST be inlined in <head> BEFORE %sveltekit.head% to prevent
 * flash of wrong theme on page load.
 * 
 * IMPORTANT: This is the canonical source. When updating, also update:
 *   - packages/desktop/src/app.html
 *   - packages/web/src/app.html
 * 
 * Three-state theme cycle: light → dark → auto (system preference)
 * Storage key: 'theme'
 * Values: 'light' | 'dark' | null (auto)
 */
(function () {
  var THEME_KEY = 'theme';
  var DARK_BG = '#222436';
  var LIGHT_BG = '#e1e2e7';

  function getSystemTheme() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  function getEffectiveTheme() {
    try {
      var stored = localStorage.getItem(THEME_KEY);
      if (stored === 'light' || stored === 'dark') return stored;
    } catch (e) {}
    return getSystemTheme();
  }

  var theme = getEffectiveTheme();
  var isDark = theme === 'dark';
  var root = document.documentElement;

  root.classList.toggle('dark', isDark);
  root.style.colorScheme = isDark ? 'dark' : 'light';
  root.style.backgroundColor = isDark ? DARK_BG : LIGHT_BG;
})();
