/**
 * Canonical source. Update these files when changing:
 *   - packages/desktop/src/app.html
 *   - packages/web/src/app.html
 *
 * Must be inlined in <head> BEFORE %sveltekit.head%
 * Storage key: 'theme' ('light' | 'dark' | null for auto)
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
