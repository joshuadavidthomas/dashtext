<script lang="ts">
  import * as Sidebar from '../ui/sidebar';
  import { getDraftsState } from '../../stores';

  const draftsState = getDraftsState();
</script>

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
      {#each draft.previewLines as line, i (`${draft.id}-${i}`)}
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
