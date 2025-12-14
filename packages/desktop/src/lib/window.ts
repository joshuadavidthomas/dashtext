import { tick } from 'svelte';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

/**
 * Shows the current window after Svelte has committed DOM updates.
 * Call this from onMount in each window's layout component.
 * Includes error handling to ensure window always shows (avoiding invisible app).
 */
export async function showWindowWhenReady(): Promise<void> {
	try {
		await tick();
		const win = getCurrentWebviewWindow();
		await win.show();
		await win.setFocus();
	} catch (err) {
		console.error('Failed to show window:', err);
		// Always show window on error to avoid permanently invisible app
		try {
			await tick();
			const win = getCurrentWebviewWindow();
			await win.show();
		} catch (fallbackErr) {
			console.error('Failed to show window even in fallback:', fallbackErr);
		}
	}
}
