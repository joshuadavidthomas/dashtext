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

<Sidebar.Provider>
  <div data-layout="root">
    <WinBar />
    <Sidebar.Root collapsible="offcanvas">
      <Sidebar.Content class="gap-0">
        {#if draftsState.isLoading}
          <div class="p-4 text-center text-sm text-sidebar-foreground/70">Loading...</div>
        {:else if draftsState.error}
          <div class="p-4 text-center text-sm text-destructive">{draftsState.error}</div>
        {:else}
          {#each draftsState.drafts as draft (draft.id)}
            <button
              onclick={() => draftsState.selectDraft(draft.id)}
              class="flex flex-col items-start gap-1 w-full px-3 py-2 transition-colors hover:bg-sidebar-accent"
              class:bg-sidebar-accent={draftsState.currentDraft?.id === draft.id}>
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
            </button>
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
  </div>
</Sidebar.Provider>
