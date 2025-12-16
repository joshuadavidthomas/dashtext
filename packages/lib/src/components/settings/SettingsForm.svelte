<script lang="ts">
	import { Switch } from '../ui/switch';
	import { getSettingsState } from '../../stores';
	import { getPlatform } from '../../platform';
	import { onMount } from 'svelte';

	const settings = getSettingsState();
	const platform = getPlatform();

	let captureShortcut = $state('CommandOrControl+Shift+C');
	let shortcutError = $state<string | null>(null);

	// Desktop-only: Load capture shortcut from database
	onMount(async () => {
		if (platform.platform === 'desktop') {
			try {
				// Dynamic import for desktop-only module
				// @ts-ignore - desktop-only import, not available in lib/web packages
				const settingsModule = await import('$lib/api/settings').catch(() => null);
				if (settingsModule) {
					const appSettings = await settingsModule.loadSettings();
					captureShortcut = appSettings.captureShortcut;
				}
			} catch (err) {
				console.error('Failed to load capture shortcut:', err);
			}
		}
	});

	async function updateCaptureShortcut() {
		if (platform.platform !== 'desktop') return;

		shortcutError = null;
		try {
			// Dynamic import for desktop-only module
			// @ts-ignore - desktop-only import, not available in lib/web packages
			const settingsModule = await import('$lib/api/settings').catch(() => null);
			if (settingsModule) {
				await settingsModule.saveCaptureShortcut(captureShortcut);
			}
		} catch (err) {
			shortcutError = err instanceof Error ? err.message : 'Failed to update shortcut';
			console.error('Failed to update capture shortcut:', err);
		}
	}
</script>

<div class="space-y-6">
	<div class="space-y-4">
		<h3 class="text-sm font-medium text-[var(--cm-foreground)]">Editor</h3>

		<div class="flex items-center justify-between">
			<div class="space-y-0.5">
				<label for="vim-switch" class="text-sm font-medium text-[var(--cm-foreground)]">
					Vim Mode
				</label>
				<p class="text-xs text-[var(--cm-comment)]">Enable vim keybindings in the editor</p>
			</div>
			<Switch
				id="vim-switch"
				checked={settings.vimEnabled}
				onCheckedChange={(v: boolean) => settings.setVimEnabled(v)}
				class="data-[state=checked]:bg-[var(--cm-accent)]"
			/>
		</div>
	</div>

	{#if platform.platform === 'desktop'}
		<div class="space-y-4">
			<h3 class="text-sm font-medium text-[var(--cm-foreground)]">Capture</h3>

			<div class="space-y-2">
				<label for="capture-shortcut" class="text-sm font-medium text-[var(--cm-foreground)]">
					Quick Capture Shortcut
				</label>
				<p class="text-xs text-[var(--cm-comment)]">
					Global keyboard shortcut to open quick capture window
				</p>
				<div class="flex gap-2">
					<input
						id="capture-shortcut"
						type="text"
						bind:value={captureShortcut}
						onblur={updateCaptureShortcut}
						class="flex-1 rounded bg-[var(--cm-background)] border border-[var(--cm-selection)] px-3 py-2 text-sm text-[var(--cm-foreground)] focus:outline-none focus:ring-1 focus:ring-[var(--cm-accent)]"
						placeholder="CommandOrControl+Shift+C"
					/>
				</div>
				{#if shortcutError}
					<p class="text-xs text-red-500">{shortcutError}</p>
				{/if}
				<p class="text-xs text-[var(--cm-comment)]">
					Format: CommandOrControl+Shift+C (Command on macOS, Ctrl elsewhere)
				</p>
			</div>
		</div>
	{/if}
</div>
