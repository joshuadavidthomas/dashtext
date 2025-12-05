import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export { default as CaptureEditor } from './CaptureEditor.svelte';

export async function openQuickCapture() {
  const existing = await WebviewWindow.getByLabel('capture');
  if (existing) {
    await existing.setFocus();
    return existing;
  }

  return new WebviewWindow('capture', {
    url: '/capture',
    title: 'Quick Capture',
    width: 600,
    height: 300,
    decorations: false,
    alwaysOnTop: true,
    center: true,
    focus: true,
    resizable: true,
    skipTaskbar: true,
  });
}
