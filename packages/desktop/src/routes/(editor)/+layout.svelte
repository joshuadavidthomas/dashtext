<script lang="ts">
	import { createEditorContext } from '@dashtext/lib/editor';
	import * as Sidebar from '@dashtext/lib/sidebar';
	import { listDrafts, createDraft, saveDraft } from '$lib/api';
	import { createDraftsState, type DraftsAPI } from '@dashtext/lib/stores';
	import { createUpdaterState } from '$lib/stores/updater.svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { replaceState } from '$app/navigation';
	import StatusLine from './StatusLine.svelte';
	import WinBar from './WinBar.svelte';
	import { UpdateDialog } from '$lib/components/updater';

	let { data, children } = $props();

	createEditorContext();

	// Create the API adapter for DraftsState
	const draftsAPI: DraftsAPI = {
		createDraft: async () => createDraft(),
		saveDraft: async (id, content) => saveDraft(id, content),
		replaceUrl: (url) => replaceState(url, {})
	};

	const draftsState = createDraftsState(() => data.drafts, draftsAPI);

	// Initialize updater
	const updater = createUpdaterState();
	$effect(() => {
		updater.init();
	});

	// Refresh drafts when window gains focus (e.g., after using quick capture)
	$effect(() => {
		let unlisten: (() => void) | null = null;

		getCurrentWindow()
			.onFocusChanged(({ payload: focused }) => {
				if (focused) {
					listDrafts().then((fresh) => {
						draftsState.drafts = fresh;
					});
				}
			})
			.then((fn) => {
				unlisten = fn;
			});

		return () => {
			unlisten?.();
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
