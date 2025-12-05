<script lang="ts">
  import '../../app.css';
  import { createEditorContext } from '$lib/components/editor';

  let { children } = $props();

  createEditorContext();
</script>

{@render children()}
