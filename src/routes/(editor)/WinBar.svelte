<script lang="ts">
	import * as AppBar from '$lib/components/appbar';
	import { Button } from '$lib/components/ui/button/index.js';
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';
	import * as Tooltip from '$lib/components/ui/tooltip/index.js';
	import { deleteDraft } from '$lib/api';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import { goto } from '$app/navigation';
	import { Minus, PanelLeft, Plus, Square, Trash2, X } from '@lucide/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	const appWindow = getCurrentWindow();
	const sidebar = useSidebar();
	const draftsState = getDraftsState();

	const currentDraftId = $derived(draftsState.currentDraft?.id ?? null);

	async function minimize() {
		await appWindow.minimize();
	}

	async function toggleMaximize() {
		await appWindow.toggleMaximize();
	}

	async function close() {
		await appWindow.close();
	}

	function handleNew() {
		goto('/drafts/new');
	}

	async function handleDelete() {
		if (!currentDraftId) return;
		await deleteDraft(currentDraftId);

		// Update local state
		draftsState.drafts = draftsState.drafts.filter((d) => d.id !== currentDraftId);

		// Navigate to another draft or new
		if (draftsState.drafts.length > 0) {
			goto(`/drafts/${draftsState.drafts[0].id}`);
		} else {
			goto('/drafts/new');
		}
	}
</script>

<Tooltip.Provider delayDuration={300}>
	<AppBar.Root as="header" data-layout="menu-bar" data-tauri-drag-region class="gap-1 bg-[var(--cm-background-dark)] p-1">
			<AppBar.Section style="-webkit-app-region: no-drag;">
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
								onclick={handleNew}
								aria-label="New draft"
							>
								<Plus class="size-3.5" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">New draft (Ctrl+N)</Tooltip.Content>
				</Tooltip.Root>

				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="toolbar"
								size="icon-sm"
								onclick={handleDelete}
								disabled={!currentDraftId}
								aria-label="Delete draft"
							>
								<Trash2 class="size-3.5" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content side="bottom">Delete draft</Tooltip.Content>
				</Tooltip.Root>
			</AppBar.Section>

			<AppBar.Section style="-webkit-app-region: no-drag;">
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
			</AppBar.Section>
	</AppBar.Root>
</Tooltip.Provider>
