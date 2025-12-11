<script lang="ts">
  import { EditorLayout } from '@dashtext/lib/editor-layout';
  import { SettingsDialog } from '@dashtext/lib';
  import { setContext } from 'svelte';

  let { data, children } = $props();

  let settingsOpen = $state(false);

  // Provide a way for children to open settings
  setContext('openSettings', () => {
    settingsOpen = true;
  });
</script>

<EditorLayout drafts={data.drafts}>
  {@render children()}
  
  {#snippet afterLayout()}
    <SettingsDialog bind:open={settingsOpen} />
  {/snippet}
</EditorLayout>
