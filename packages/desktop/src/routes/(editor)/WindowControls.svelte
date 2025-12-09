<script lang="ts">
	import { Button } from '@dashtext/lib/button';
	import * as Tooltip from '@dashtext/lib/tooltip';
	import { Minus, Square, X } from '@lucide/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	const appWindow = getCurrentWindow();

	async function minimize() {
		await appWindow.minimize();
	}

	async function toggleMaximize() {
		await appWindow.toggleMaximize();
	}

	async function close() {
		await appWindow.close();
	}
</script>

<Tooltip.Root>
	<Tooltip.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="toolbar-minimize"
				size="icon-sm"
				onclick={minimize}
				aria-label="Minimize"
			>
				<Minus class="size-3.5" />
			</Button>
		{/snippet}
	</Tooltip.Trigger>
	<Tooltip.Content side="bottom">Minimize</Tooltip.Content>
</Tooltip.Root>

<Tooltip.Root>
	<Tooltip.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="toolbar-maximize"
				size="icon-sm"
				onclick={toggleMaximize}
				aria-label="Maximize"
			>
				<Square class="size-3" />
			</Button>
		{/snippet}
	</Tooltip.Trigger>
	<Tooltip.Content side="bottom">Maximize</Tooltip.Content>
</Tooltip.Root>

<Tooltip.Root>
	<Tooltip.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="toolbar-close"
				size="icon-sm"
				onclick={close}
				aria-label="Close"
			>
				<X class="size-3.5" />
			</Button>
		{/snippet}
	</Tooltip.Trigger>
	<Tooltip.Content side="bottom">Close</Tooltip.Content>
</Tooltip.Root>
