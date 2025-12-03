<script lang="ts">
	import type { Draft } from '$lib/api/drafts';

	let {
		draft,
		isActive,
		onclick
	}: {
		draft: Draft;
		isActive: boolean;
		onclick: () => void;
	} = $props();

	/**
	 * Extract title from first line of content
	 */
	function getTitle(content: string): string {
		const firstLine = content.split('\n')[0].trim();
		return firstLine || 'Untitled';
	}

	/**
	 * Extract up to 3 preview lines (after the title)
	 */
	function getPreviewLines(content: string): string[] {
		const lines = content.split('\n').slice(1);
		const nonEmpty = lines.filter((line) => line.trim());
		return nonEmpty.slice(0, 3);
	}

	/**
	 * Format timestamp as readable date/time
	 */
	function formatTimestamp(iso: string): string {
		const date = new Date(iso);
		return date.toLocaleString(undefined, {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}
</script>

<button
	{onclick}
	class="w-full px-3 py-2 text-left transition-colors hover:bg-sidebar-accent"
	class:bg-sidebar-accent={isActive}
>
	<div class="truncate text-sm font-medium text-sidebar-foreground">
		{getTitle(draft.content)}
	</div>
	{#each getPreviewLines(draft.content) as line}
		<div class="truncate text-xs text-sidebar-foreground/60">
			{line}
		</div>
	{/each}
	<div class="mt-1 text-xs text-sidebar-foreground/40">
		{formatTimestamp(draft.modified_at)}
	</div>
</button>
