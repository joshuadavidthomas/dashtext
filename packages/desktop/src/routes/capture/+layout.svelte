<script lang="ts">
  import '../../app.css';
  import { createEditorContext } from '@dashtext/lib/editor';

  let { children } = $props();

  createEditorContext();
</script>

{@render children()}
