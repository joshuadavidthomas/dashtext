<script lang="ts">
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import { getUIState } from '$lib/stores/ui.svelte';

	const draftsState = getDraftsState();
	const uiState = getUIState();

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

{#if uiState.sidebarVisible}
<aside data-layout="sidebar">
	<div class="sidebar-header">
		<button onclick={() => draftsState.newDraft()} class="new-draft-btn"> + New </button>
	</div>

	{#if draftsState.isLoading}
		<div class="loading">Loading...</div>
	{:else if draftsState.error}
		<div class="error">{draftsState.error}</div>
	{:else}
		<ul class="draft-list">
			{#each draftsState.drafts as draft (draft.id)}
				<li>
					<button
						class="draft-item"
						class:selected={draftsState.currentDraft?.id === draft.id}
						onclick={() => draftsState.selectDraft(draft.id)}
					>
						{getDisplayTitle(draft.content)}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</aside>
{/if}

<style>
	.sidebar-header {
		padding: calc(var(--spacing) * 2);
		border-bottom: 1px solid var(--cm-gutter-foreground);
	}

	.new-draft-btn {
		width: 100%;
		padding: calc(var(--spacing) * 2);
		background: var(--cm-accent);
		color: var(--cm-accent-foreground);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: 500;
	}

	.new-draft-btn:hover {
		opacity: 0.9;
	}

	.draft-list {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.draft-item {
		width: 100%;
		padding: calc(var(--spacing) * 3);
		text-align: left;
		background: none;
		border: none;
		border-bottom: 1px solid var(--cm-gutter-foreground);
		cursor: pointer;
		color: var(--cm-foreground);
		font-size: 0.875rem;
	}

	.draft-item:hover {
		background: var(--cm-background-highlight);
	}

	.draft-item.selected {
		background: var(--cm-selection);
	}

	.loading,
	.error {
		padding: calc(var(--spacing) * 4);
		text-align: center;
		color: var(--cm-foreground);
	}

	.error {
		color: var(--cm-error);
	}
</style>
