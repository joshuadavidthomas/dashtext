import { tick } from 'svelte';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

/**
 * Shows the current window after Svelte has committed DOM updates.
 * Call this from onMount in each window's layout component.
 */
export async function showWindowWhenReady(): Promise<void> {
	await tick();
	const win = getCurrentWebviewWindow();
	await win.show();
	await win.setFocus();
}
