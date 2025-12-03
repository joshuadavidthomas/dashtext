<script lang="ts">
	import { getDraftsState } from '$lib/stores/drafts.svelte';

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

<style>
	.sidebar-header {
		padding: 0.5rem;
		border-bottom: 1px solid var(--border, #333);
	}

	.new-draft-btn {
		width: 100%;
		padding: 0.5rem;
		background: var(--accent, #4a9eff);
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
	}

	.draft-list {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.draft-item {
		width: 100%;
		padding: 0.75rem;
		text-align: left;
		background: none;
		border: none;
		border-bottom: 1px solid var(--border, #333);
		cursor: pointer;
		color: inherit;
	}

	.draft-item:hover {
		background: var(--hover, #2a2a2a);
	}

	.draft-item.selected {
		background: var(--selected, #3a3a3a);
	}

	.loading,
	.error {
		padding: 1rem;
		text-align: center;
	}

	.error {
		color: var(--error, #ff4a4a);
	}
</style>
