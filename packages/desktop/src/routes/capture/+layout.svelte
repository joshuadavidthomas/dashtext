<script lang="ts">
  import '../../app.css';
  import { createEditorContext } from '@dashtext/lib/editor';
  import { onMount } from 'svelte';
  import { showWindowWhenReady } from '$lib/window';

  let { children } = $props();

  createEditorContext();

  onMount(() => {
    showWindowWhenReady();
  });
</script>

{@render children()}
