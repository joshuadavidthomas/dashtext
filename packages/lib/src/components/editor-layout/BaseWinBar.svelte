<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as AppBar from '../appbar';
  import { Button } from '../ui/button';
  import { useSidebar } from '../ui/sidebar';
  import * as Tooltip from '../ui/tooltip';
  import { getDraftsState } from '../../stores';
  import { getPlatform } from '../../platform';
  import { goto } from '$app/navigation';
  import { getContext } from 'svelte';
  import { PanelLeft, Plus, Trash2, Zap, Settings, Minus, Square, X } from '@lucide/svelte';

  type Props = {
    /** Additional toolbar content */
    children?: Snippet;
  };

  let { children }: Props = $props();

  const platform = getPlatform();
  const sidebar = useSidebar();
  const draftsState = getDraftsState();
  const openSettingsWeb = getContext<(() => void) | undefined>('openSettings');

  function handleNew() {
    goto('/drafts/new');
  }

  async function handleDelete() {
    const draftsState = getDraftsState();
    await draftsState.deleteCurrentDraft();
  }

  async function handleQuickCapture() {
    await platform.quickCapture?.open();
  }

  async function handleSettings() {
    // Desktop: open window via platform API
    if (platform.settings) {
      await platform.settings.open();
    } else {
      // Web: trigger via context
      openSettingsWeb?.();
    }
  }

  // Keyboard shortcut for settings (Ctrl+, or Cmd+,)
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === ',' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSettings();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<AppBar.Root
  as="header"
  data-layout="menu-bar"
  data-tauri-drag-region={platform.platform === 'desktop' ? true : undefined}
  class="gap-1 bg-[var(--cm-background-dark)] p-1"
>
  <AppBar.Section style={platform.platform === 'desktop' ? '-webkit-app-region: no-drag;' : ''}>
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
            disabled={draftsState.currentDraft === null}
            aria-label="Delete draft"
          >
            <Trash2 class="size-3.5" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content side="bottom">Delete draft</Tooltip.Content>
    </Tooltip.Root>

    {#if platform.quickCapture}
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
    {/if}

    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button
            {...props}
            variant="toolbar"
            size="icon-sm"
            onclick={handleSettings}
            aria-label="Settings"
          >
            <Settings class="size-3.5" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content side="bottom">Settings (Ctrl+,)</Tooltip.Content>
    </Tooltip.Root>

    {#if children}{@render children()}{/if}
  </AppBar.Section>

  {#if platform.window}
    <AppBar.Section style="-webkit-app-region: no-drag;">
      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button
              {...props}
              variant="toolbar-minimize"
              size="icon-sm"
              onclick={() => platform.window?.minimize()}
              aria-label="Minimize"
            >
              <Minus class="size-3.5" />
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content side="bottom">Minimize</Tooltip.Content>
      </Tooltip.Root>

      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button
              {...props}
              variant="toolbar-maximize"
              size="icon-sm"
              onclick={() => platform.window?.maximize()}
              aria-label="Maximize"
            >
              <Square class="size-3" />
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content side="bottom">Maximize</Tooltip.Content>
      </Tooltip.Root>

      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button
              {...props}
              variant="toolbar-close"
              size="icon-sm"
              onclick={() => platform.window?.close()}
              aria-label="Close"
            >
              <X class="size-3.5" />
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content side="bottom">Close</Tooltip.Content>
      </Tooltip.Root>
    </AppBar.Section>
  {/if}
</AppBar.Root>
