<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import DraftListItem from './draft-list-item.svelte';

	const draftsState = getDraftsState();
</script>

<Sidebar.Root collapsible="offcanvas">
	<Sidebar.Content>
		{#if draftsState.isLoading}
			<div class="p-4 text-center text-sm text-sidebar-foreground/70">Loading...</div>
		{:else if draftsState.error}
			<div class="p-4 text-center text-sm text-destructive">{draftsState.error}</div>
		{:else}
			{#each draftsState.drafts as draft (draft.id)}
				<DraftListItem
					{draft}
					isActive={draftsState.currentDraft?.id === draft.id}
					onclick={() => draftsState.selectDraft(draft.id)}
				/>
			{/each}
		{/if}
	</Sidebar.Content>
</Sidebar.Root>
