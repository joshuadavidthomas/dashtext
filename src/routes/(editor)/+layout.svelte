<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { createEditorContext } from '$lib/components/editor';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { UpdateDialog } from '$lib/components/updater';
	import { listDrafts } from '$lib/api';
	import { createDraftsState } from '$lib/stores/drafts.svelte';
	import { createUpdaterState } from '$lib/stores/updater.svelte';
	import StatusLine from './StatusLine.svelte';
	import WinBar from './WinBar.svelte';

	let { data, children } = $props();

	createEditorContext();
	const draftsState = createDraftsState(() => data.drafts);
	const updater = createUpdaterState();

	$effect(() => {
		updater.init();
	});

	// Refresh drafts when window gains focus (e.g., after using quick capture)
	$effect(() => {
		const unlistenPromise = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
			if (focused) {
				listDrafts().then((fresh) => {
					draftsState.drafts = fresh;
				});
			}
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	});
</script>

<Sidebar.Provider>
	<div data-layout="root">
		<WinBar />
		<Sidebar.Root collapsible="offcanvas">
			<Sidebar.Content class="gap-0">
				{#if draftsState.currentDraft === null}
					<a
						href="/drafts/new"
						class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors bg-sidebar-accent"
					>
						<div class="truncate text-sm font-medium text-sidebar-foreground">
							Untitled
						</div>
					</a>
				{/if}
				{#each draftsState.drafts as draft (draft.id)}
					<a
						href="/drafts/{draft.id}"
						class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors hover:bg-sidebar-accent"
						class:bg-sidebar-accent={draftsState.currentDraft?.id === draft.id}
					>
					<div class="w-full truncate text-sm font-medium text-sidebar-foreground">
						{draft.title}
					</div>
					{#each draft.previewLines as line, i (i)}
						<div class="w-full truncate text-xs text-sidebar-foreground/60">
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
	<UpdateDialog />
</Sidebar.Provider>
