<script lang="ts">
  import * as Tooltip from '@dashtext/ui/tooltip';
  import { getUpdaterState } from '$lib/stores/updater.svelte';

  const updater = getUpdaterState();

  const tooltipText = $derived.by(() => {
    switch (updater.status) {
      case 'downloading':
        return 'Downloading update...';
      case 'ready':
        return 'Update ready. Click to restart.';
      case 'available':
        return `Update available: v${updater.currentVersion} → v${updater.newVersion}`;
      default:
        return null;
    }
  });
</script>

{#if tooltipText}
  <Tooltip.Provider>
    <Tooltip.Root>
      <Tooltip.Trigger>
        <button
          type="button"
          class="flex items-center gap-1 px-2 py-0.5 text-xs text-[var(--cm-statusline-fg)] hover:text-[var(--cm-foreground)] transition-colors"
          onclick={() => updater.openDialog()}
        >
          <span>v{updater.currentVersion}</span>
          {#if updater.hasUpdate}
            <span class="size-1.5 rounded-full bg-green-500"></span>
          {/if}
        </button>
      </Tooltip.Trigger>
      <Tooltip.Content>
        {tooltipText}
      </Tooltip.Content>
    </Tooltip.Root>
  </Tooltip.Provider>
{:else}
  <span class="flex items-center gap-1 px-2 py-0.5 text-xs text-[var(--cm-statusline-fg)]">
    v{updater.currentVersion}
  </span>
{/if}
