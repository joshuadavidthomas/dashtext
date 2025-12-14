<script lang="ts">
  import { EditorLayout } from '@dashtext/lib/editor-layout';
  import { createUpdaterState } from '$lib/stores/updater.svelte';
  import { UpdateDialog, VersionIndicator } from '$lib/components/updater';
  import { onMount } from 'svelte';
  import { showWindowWhenReady } from '$lib/window';

  let { data, children } = $props();

  // Initialize updater (desktop-only)
  const updater = createUpdaterState();
  $effect(() => {
    updater.init();
  });

  onMount(() => {
    showWindowWhenReady();
  });
</script>

<EditorLayout drafts={data.drafts}>
  {#snippet statusExtra()}
    <VersionIndicator />
  {/snippet}

  {#snippet afterLayout()}
    <UpdateDialog />
  {/snippet}

  {@render children()}
</EditorLayout>
