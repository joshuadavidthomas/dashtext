import { createContext } from 'svelte';
import type { UnlistenFn } from '@tauri-apps/api/event';

export type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';

/** Update info returned from Rust backend */
interface UpdateInfo {
	current_version: string;
	new_version: string;
	release_notes: string | null;
	download_url: string;
	can_auto_update: boolean;
}

export class UpdaterState {
	status = $state<UpdateStatus>('idle');
	currentVersion = $state(__APP_VERSION__);
	canAutoUpdate = $state(false);
	downloadProgress = $state(0);
	error = $state<string | null>(null);
	dialogOpen = $state(false);

	// Update info from backend
	private updateInfo = $state<UpdateInfo | null>(null);
	private checkInterval: ReturnType<typeof setInterval> | null = null;
	private unlistenProgress: UnlistenFn | null = null;

	newVersion = $derived(this.updateInfo?.new_version ?? null);
	releaseNotes = $derived(this.updateInfo?.release_notes ?? null);
	hasUpdate = $derived(this.status === 'available' || this.status === 'ready');

	async init() {
		const { invoke } = await import('@tauri-apps/api/core');
		const { listen } = await import('@tauri-apps/api/event');

		try {
			// Get current version from backend
			this.currentVersion = await invoke<string>('get_current_version');
			// Check if we can auto-update
			this.canAutoUpdate = await invoke<boolean>('can_auto_update');
			// Check for updates
			await this.checkForUpdate();

			// Listen for download progress events
			this.unlistenProgress = await listen<{
				downloaded: number;
				total: number | null;
				percent: number | null;
			}>('update-progress', (event) => {
				this.downloadProgress = event.payload.percent ?? 0;
			});
		} catch (e) {
			console.debug('Updater init failed:', e);
		}

		// Check for updates every hour
		this.checkInterval = setInterval(
			() => this.checkForUpdate(),
			60 * 60 * 1000 // 1 hour in milliseconds
		);
	}

	async checkForUpdate() {
		// Don't re-check if we already have an update available
		if (this.status === 'available' || this.status === 'downloading' || this.status === 'ready') {
			return;
		}

		this.status = 'checking';
		this.error = null;

		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const update = await invoke<UpdateInfo | null>('check_update');
			if (update) {
				this.updateInfo = update;
				this.canAutoUpdate = update.can_auto_update;
				this.status = 'available';
			} else {
				this.status = 'idle';
			}
		} catch (e) {
			console.debug('Update check failed:', e);
			this.status = 'idle';
		}
	}

	async downloadAndInstall() {
		if (!this.updateInfo || !this.canAutoUpdate) return;

		this.status = 'downloading';
		this.downloadProgress = 0;
		this.error = null;

		try {
			const { invoke } = await import('@tauri-apps/api/core');
			// Note: Our backend doesn't report progress yet, so we just show indeterminate
			// Could be enhanced to use Tauri events for progress reporting
			await invoke('download_and_install_update');
			this.downloadProgress = 100;
			this.status = 'ready';
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
			this.status = 'error';
		}
	}

	async restart() {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			await invoke('restart_app');
		} catch (e) {
			// restart_app calls exec() which replaces the process
			// If we get here, something went wrong
			this.error = e instanceof Error ? e.message : String(e);
		}
	}

	openDialog() {
		this.dialogOpen = true;
	}

	closeDialog() {
		this.dialogOpen = false;
	}
}

export const [getUpdaterState, setUpdaterState] = createContext<UpdaterState>();

export function createUpdaterState(): UpdaterState {
	const state = new UpdaterState();
	setUpdaterState(state);
	return state;
}
