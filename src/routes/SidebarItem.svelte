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

	function getTitle(content: string): string {
		const firstLine = content.split('\n')[0].trim();
		return firstLine || 'Untitled';
	}

	function getPreviewLines(content: string): string[] {
		const lines = content.split('\n').slice(1);
		const nonEmpty = lines.filter((line) => line.trim());
		return nonEmpty.slice(0, 3);
	}

	function formatTimestamp(value: string): string {
		const asNumber = parseInt(value);
		const date =
			!isNaN(asNumber) && value === String(asNumber)
				? new Date(asNumber * 1000) // Unix timestamp (seconds)
				: new Date(value); // ISO/RFC 3339 string

		if (isNaN(date.getTime())) return 'Unknown date';

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
	class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors hover:bg-sidebar-accent"
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
	{#if draft.content.trim()}
		<div class="text-xs text-sidebar-foreground/40">
			{formatTimestamp(draft.modified_at)}
		</div>
	{/if}
</button>
