<script lang="ts">
  import { Editor } from '@dashtext/lib/editor';
  import { getDraftsState } from '@dashtext/lib/stores';
  import { beforeNavigate } from '$app/navigation';
  import { getContext } from 'svelte';

  let { data } = $props();

  const draftsState = getDraftsState();

  $effect.pre(() => {
    draftsState.setCurrentDraft(data.draft);
  });

  beforeNavigate(async () => {
    const navigationHandlers = getContext<{handleDraftSave: () => Promise<void>, handleDraftDelete: () => Promise<void>, handleDraftArchive: () => Promise<void>}>('navigationHandlers');
    await navigationHandlers.handleDraftSave();
  });
</script>

<Editor />
