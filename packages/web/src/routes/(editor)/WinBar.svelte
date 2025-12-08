<script lang="ts">
  import { goto } from '$app/navigation';
  import { deleteDraft } from '$lib/api';
  import * as AppBar from '@dashtext/lib/appbar';
  import { Button } from '@dashtext/lib/button';
  import { useSidebar } from '@dashtext/lib/sidebar';
  import { getDraftsState } from '@dashtext/lib/stores';
  import * as Tooltip from '@dashtext/lib/tooltip';
  import { PanelLeft, Plus, Trash2, Zap } from '@lucide/svelte';

  const sidebar = useSidebar();
  const draftsState = getDraftsState();

  const currentDraftId = $derived(draftsState.currentDraft?.id ?? null);

  function handleNew() {
    goto('/drafts/new');
  }

  async function handleDelete() {
    if (!currentDraftId) return;
    await deleteDraft(currentDraftId);

    draftsState.drafts = draftsState.drafts.filter((d) => d.id !== currentDraftId);

    if (draftsState.drafts.length > 0) {
      goto(`/drafts/${draftsState.drafts[0].id}`);
    } else {
      goto('/drafts/new');
    }
  }

  function handleQuickCapture() {
    goto('/capture');
  }
</script>

<Tooltip.Provider delayDuration={300}>
  <AppBar.Root as="header" data-layout="menu-bar" class="gap-1 bg-[var(--cm-background-dark)] p-1">
      <AppBar.Section>
        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="toolbar"
                size="icon-sm"
                onclick={() => sidebar.toggle()}
                aria-label="Toggle sidebar"
                aria-pressed={sidebar.open}
                class={sidebar.open ? 'text-[var(--cm-accent)]' : ''}
              >
                <PanelLeft class="size-3.5" />
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom">Toggle sidebar (Ctrl+B)</Tooltip.Content>
        </Tooltip.Root>

        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="toolbar"
                size="icon-sm"
                onclick={handleNew}
                aria-label="New draft"
              >
                <Plus class="size-3.5" />
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom">New draft (Ctrl+N)</Tooltip.Content>
        </Tooltip.Root>

        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="toolbar"
                size="icon-sm"
                onclick={handleDelete}
                disabled={!currentDraftId}
                aria-label="Delete draft"
              >
                <Trash2 class="size-3.5" />
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom">Delete draft</Tooltip.Content>
        </Tooltip.Root>

        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <Button
                {...props}
                variant="toolbar"
                size="icon-sm"
                onclick={handleQuickCapture}
                aria-label="Quick capture"
              >
                <Zap class="size-3.5" />
              </Button>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content side="bottom">Quick capture</Tooltip.Content>
        </Tooltip.Root>
      </AppBar.Section>

      <AppBar.Section>
        <!-- No window controls on web -->
      </AppBar.Section>
  </AppBar.Root>
</Tooltip.Provider>
