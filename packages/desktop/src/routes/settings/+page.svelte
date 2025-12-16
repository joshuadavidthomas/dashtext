<script lang="ts">
	import { SettingsForm } from '@dashtext/lib';
	import { Button } from '@dashtext/lib/button';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { X } from '@lucide/svelte';

	// Double-escape to close settings window
	$effect(() => {
		let lastEscape = 0;

		function handleKeydown(event: KeyboardEvent) {
			if (event.key === 'Escape') {
				const now = Date.now();
				if (now - lastEscape < 500) {
					getCurrentWindow().close();
				}
				lastEscape = now;
			}
		}

		document.addEventListener('keydown', handleKeydown);
		return () => document.removeEventListener('keydown', handleKeydown);
	});
</script>

<div class="flex flex-col h-screen bg-[var(--cm-background)]">
	<!-- Custom titlebar matching main window -->
	<header
		data-tauri-drag-region
		class="flex items-center justify-between gap-1 p-1 bg-[var(--cm-background-dark)]"
	>
		<h1 class="text-sm font-medium px-2 text-[var(--cm-foreground)]">Settings</h1>
		<Button
			variant="toolbar-close"
			size="icon-sm"
			onclick={() => getCurrentWindow().close()}
			style="-webkit-app-region: no-drag;"
			aria-label="Close"
		>
			<X class="size-3.5" />
		</Button>
	</header>

	<main class="flex-1 overflow-auto p-6">
		<SettingsForm />
	</main>
</div>
