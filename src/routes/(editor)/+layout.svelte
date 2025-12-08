<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { listDrafts } from '$lib/api';
	import { createDraftsState } from '$lib/stores/drafts.svelte';
	import { createUpdaterState } from '$lib/stores/updater.svelte';
	import { isTauri } from '$lib/platform';
	import StatusLine from './StatusLine.svelte';
	import WinBar from './WinBar.svelte';

	let { data, children } = $props();

	const inTauri = isTauri();

	const draftsState = createDraftsState(() => data.drafts);

	// Create updater context synchronously (safe - no Tauri imports at module level)
	// Only initialize in Tauri mode
	const updater = inTauri ? createUpdaterState() : null;

	// Initialize updater async (Tauri APIs are dynamically imported inside init())
	$effect(() => {
		updater?.init();
	});

	// Refresh drafts when window gains focus (e.g., after using quick capture)
	// On web, use visibilitychange event instead
	$effect(() => {
		if (inTauri) {
			let unlisten: (() => void) | null = null;

			import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
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
			});

			return () => {
				unlisten?.();
			};
		} else {
			// Web: refresh on visibility change
			const handleVisibility = () => {
				if (document.visibilityState === 'visible') {
					listDrafts().then((fresh) => {
						draftsState.drafts = fresh;
					});
				}
			};

			document.addEventListener('visibilitychange', handleVisibility);
			return () => {
				document.removeEventListener('visibilitychange', handleVisibility);
			};
		}
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
	{#if inTauri}
		{#await import('$lib/components/updater') then { UpdateDialog }}
			<UpdateDialog />
		{/await}
	{/if}
</Sidebar.Provider>
