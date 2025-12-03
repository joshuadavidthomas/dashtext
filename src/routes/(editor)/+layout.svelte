<script lang="ts">
	import { createEditorContext } from '$lib/components/editor';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import StatusLine from './StatusLine.svelte';
	import WinBar from './WinBar.svelte';

	let { data, children } = $props();

	createEditorContext();
</script>

<Sidebar.Provider>
	<div data-layout="root">
		<WinBar />
		<Sidebar.Root collapsible="offcanvas">
			<Sidebar.Content class="gap-0">
				{#each data.drafts as draft (draft.id)}
					<a
						href="/drafts/{draft.id}"
						class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors hover:bg-sidebar-accent"
					>
						<div class="truncate text-sm font-medium text-sidebar-foreground">
							{draft.title}
						</div>
						{#each draft.previewLines as line}
							<div class="truncate text-xs text-sidebar-foreground/60">
								{line}
							</div>
						{/each}
						{#if draft.content.trim()}
							<div class="text-xs text-sidebar-foreground/40">
								{draft.formattedModifiedAt}
							</div>
						{/if}
					</a>
				{/each}
			</Sidebar.Content>
		</Sidebar.Root>
		<main data-layout="main">
			{@render children()}
		</main>
		<aside data-layout="aside">
			<!-- Future: aside content -->
		</aside>
		<StatusLine />
	</div>
</Sidebar.Provider>
