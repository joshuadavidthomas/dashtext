<script lang="ts">
  import { EditorLayout } from '@dashtext/lib/editor-layout';

  let { data, children } = $props();
</script>

<EditorLayout drafts={data.drafts}>
  {@render children()}
</EditorLayout>
