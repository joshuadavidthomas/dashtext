<script lang="ts">
	import '../app.css';
	import { setPlatformContext } from '@dashtext/lib/platform';
	import { createSettingsContext, OnboardingDialog } from '@dashtext/lib';
	import { desktopPlatform } from '$lib/platform';
	import { openQuickCapture } from '$lib/components/capture';
	import { initializeCaptureShortcut } from '$lib/api/settings';
	import { listen } from '@tauri-apps/api/event';

	let { children } = $props();

	setPlatformContext(desktopPlatform);
	createSettingsContext();

	// Initialize capture shortcut on startup
	$effect(() => {
		initializeCaptureShortcut().catch(err => {
			console.error('Failed to initialize capture shortcut:', err);
		});
	});

	// Listen for global hotkey events
	$effect(() => {
		const unlisten = listen('hotkey:capture', async () => {
			console.log('Global hotkey triggered, opening quick capture');
			try {
				await openQuickCapture();
			} catch (e) {
				console.error('Failed to open quick capture:', e);
			}
		});

		return () => {
			unlisten.then(fn => fn());
		};
	});
</script>

<OnboardingDialog />
{@render children()}
