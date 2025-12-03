<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Minus, Square, X, PanelLeft, Plus } from '@lucide/svelte';
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import * as Tooltip from '$lib/components/ui/tooltip/index.js';

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
		class="flex h-[var(--layout-menu-h)] items-center justify-between bg-[var(--cm-background-dark)] px-2"
	>
		<div class="flex items-center gap-1" style="-webkit-app-region: no-drag;">
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<button
							{...props}
							onclick={() => sidebar.toggle()}
							class="p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
							class:text-[var(--cm-accent)]={sidebar.open}
							aria-label="Toggle sidebar"
							aria-pressed={sidebar.open}
						>
							<PanelLeft class="size-3.5" />
						</button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom">Toggle sidebar (Ctrl+B)</Tooltip.Content>
			</Tooltip.Root>

			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<button
							{...props}
							onclick={() => draftsState.newDraft()}
							class="p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
							aria-label="New draft"
						>
							<Plus class="size-3.5" />
						</button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom">New draft (Ctrl+N)</Tooltip.Content>
			</Tooltip.Root>
		</div>

		<div class="flex items-center gap-1" style="-webkit-app-region: no-drag;">
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<button
							{...props}
							onclick={minimize}
							class="p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
							aria-label="Minimize"
						>
							<Minus class="size-3.5" />
						</button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom">Minimize</Tooltip.Content>
			</Tooltip.Root>

			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<button
							{...props}
							onclick={toggleMaximize}
							class="p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
							aria-label="Maximize"
						>
							<Square class="size-3" />
						</button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom">Maximize</Tooltip.Content>
			</Tooltip.Root>

			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<button
							{...props}
							onclick={close}
							class="p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-error)]"
							aria-label="Close"
						>
							<X class="size-3.5" />
						</button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="bottom">Close</Tooltip.Content>
			</Tooltip.Root>
		</div>
	</header>
</Tooltip.Provider>
