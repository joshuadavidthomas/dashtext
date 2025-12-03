<script lang="ts">
	import { Editor, createEditorContext } from '$lib/components/editor';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import SidebarItem from './SidebarItem.svelte';
	import StatusLine from './StatusLine.svelte';
	import WinBar from './WinBar.svelte';

	const draftsState = getDraftsState();

	createEditorContext();
</script>

<WinBar />
<Sidebar.Root collapsible="offcanvas">
	<Sidebar.Content class="gap-0">
		{#if draftsState.isLoading}
			<div class="p-4 text-center text-sm text-sidebar-foreground/70">Loading...</div>
		{:else if draftsState.error}
			<div class="p-4 text-center text-sm text-destructive">{draftsState.error}</div>
		{:else}
			{#each draftsState.drafts as draft (draft.id)}
				<SidebarItem
					{draft}
					isActive={draftsState.currentDraft?.id === draft.id}
					onclick={() => draftsState.selectDraft(draft.id)}
				/>
			{/each}
		{/if}
	</Sidebar.Content>
</Sidebar.Root>
<main data-layout="main">
	<Editor />
</main>
<aside data-layout="aside">
	<!-- Future: aside content -->
</aside>
<StatusLine />
