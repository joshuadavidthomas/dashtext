<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import { Plus } from '@lucide/svelte';

	const draftsState = getDraftsState();

	/**
	 * Get display title from draft content (first line, truncated)
	 */
	function getDisplayTitle(content: string): string {
		const firstLine = content.split('\n')[0].trim();
		if (!firstLine) return 'Untitled';
		if (firstLine.length > 30) return firstLine.slice(0, 30) + '...';
		return firstLine;
	}
</script>

<Sidebar.Root collapsible="offcanvas">
	<Sidebar.Header class="p-2">
		<button
			onclick={() => draftsState.newDraft()}
			class="flex w-full items-center justify-center gap-2 rounded-md bg-sidebar-primary px-3 py-2 text-sm font-medium text-sidebar-primary-foreground hover:opacity-90"
		>
			<Plus class="size-4" />
			<span>New Draft</span>
		</button>
	</Sidebar.Header>

	<Sidebar.Content>
		{#if draftsState.isLoading}
			<div class="p-4 text-center text-sm text-sidebar-foreground/70">Loading...</div>
		{:else if draftsState.error}
			<div class="p-4 text-center text-sm text-destructive">{draftsState.error}</div>
		{:else}
			<Sidebar.Group class="p-0">
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each draftsState.drafts as draft (draft.id)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton
									isActive={draftsState.currentDraft?.id === draft.id}
									onclick={() => draftsState.selectDraft(draft.id)}
									class="rounded-none"
								>
									{getDisplayTitle(draft.content)}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/if}
	</Sidebar.Content>
</Sidebar.Root>
