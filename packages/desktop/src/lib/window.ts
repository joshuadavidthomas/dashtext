import { tick } from 'svelte';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export async function showWindowWhenReady(): Promise<void> {
	await tick();
	const win = getCurrentWebviewWindow();
	await win.show();
	await win.setFocus();
}
