<script lang="ts">
	import { Switch } from '../ui/switch';
	import HotkeyInput from './HotkeyInput.svelte';
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
				<div class="text-sm font-medium text-[var(--cm-foreground)]">
					Quick Capture Shortcut
				</div>
				<p class="text-xs text-[var(--cm-comment)]">
					Global keyboard shortcut to open quick capture window
				</p>
				<HotkeyInput
					value={captureShortcut}
					onchange={(newValue: string) => {
						captureShortcut = newValue;
						updateCaptureShortcut();
					}}
					error={shortcutError}
				/>
			</div>
		</div>
	{/if}
</div>
