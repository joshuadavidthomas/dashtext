<script lang="ts">
  import { Editor } from '@dashtext/lib/editor';
  import { getDraftsState } from '@dashtext/lib/stores';
  import { beforeNavigate } from '$app/navigation';

  let { data } = $props();

  const draftsState = getDraftsState();

  $effect.pre(() => {
    draftsState.setCurrentDraft(data.draft);
  });

  beforeNavigate(() => draftsState.flushPendingSave());
</script>

<Editor />
