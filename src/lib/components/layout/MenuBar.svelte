<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Minus, Square, X, PanelLeft, Plus } from '@lucide/svelte';
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import * as Tooltip from '$lib/components/ui/tooltip/index.js';
	import { Button } from '$lib/components/ui/button/index.js';

	const appWindow = getCurrentWindow();
	const sidebar = useSidebar();
	const draftsState = getDraftsState();

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

<Tooltip.Provider delayDuration={300}>
	<header
		data-layout="menu-bar"
		data-tauri-drag-region
		class="flex cursor-default select-none items-stretch justify-between bg-[var(--cm-background-dark)] p-1"
	>
		<div class="flex items-center gap-1" style="-webkit-app-region: no-drag;">
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="toolbar"
							size="icon-sm"
							onclick={() => sidebar.toggle()}
							aria-label="Toggle sidebar"
							aria-pressed={sidebar.open}
							class={sidebar.open ? 'text-[var(--cm-accent)]' : ''}
						>
							<PanelLeft class="size-3.5" />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom">Toggle sidebar (Ctrl+B)</Tooltip.Content>
			</Tooltip.Root>

			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="toolbar"
							size="icon-sm"
							onclick={() => draftsState.newDraft()}
							aria-label="New draft"
						>
							<Plus class="size-3.5" />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom">New draft (Ctrl+N)</Tooltip.Content>
			</Tooltip.Root>
		</div>

		<div class="flex items-center gap-1" style="-webkit-app-region: no-drag;">
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
		</div>
	</header>
</Tooltip.Provider>
