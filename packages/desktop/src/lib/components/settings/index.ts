import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export async function openSettings() {
	const existing = await WebviewWindow.getByLabel('settings');
	if (existing) {
		await existing.setFocus();
		return existing;
	}

  return new WebviewWindow('settings', {
    url: '/settings',
    title: 'Settings',
    width: 500,
    height: 400,
    decorations: false, // Custom titlebar
    center: true,
    focus: false,
    visible: false,
    backgroundColor: '#222436',
    resizable: false
  });
}
