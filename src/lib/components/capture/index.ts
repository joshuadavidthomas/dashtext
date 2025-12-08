import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export { default as CaptureEditor } from './CaptureEditor.svelte';

export async function openQuickCapture() {
  const existing = await WebviewWindow.getByLabel('capture');
  if (existing) {
    await existing.setFocus();
    return existing;
  }

  const win = new WebviewWindow('capture', {
    url: '/capture',
    title: 'Quick Capture',
    width: 600,
    height: 300,
    decorations: false,
    alwaysOnTop: true,
    center: true,
    visible: false, // Start hidden to prevent white flash
    resizable: true,
    skipTaskbar: true,
  });

  // Listen for global event when page signals it's ready (CSS loaded and painted)
  const unlisten = await listen('capture-ready', async () => {
    await win.show();
    await win.setFocus();
    unlisten(); // Clean up listener
  });

  return win;
}
