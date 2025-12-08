<script lang="ts">
  import { createDraft, saveDraft } from '$lib/api';
  import { VimModeIndicator } from '@dashtext/lib/editor';
  import { goto } from '$app/navigation';

  let saving = $state(false);
  let content = $state('');

  async function handleSubmit() {
    if (!content.trim() || saving) return;

    saving = true;
    try {
      const draft = await createDraft();
      await saveDraft(draft.id, content.trim());
      goto(`/drafts/${draft.id}`);
    } finally {
      saving = false;
    }
  }

  function handleCancel() {
    goto('/drafts/new');
  }
</script>

<div data-layout="capture-root">
  <div data-layout="capture-editor" class="p-4">
    <textarea
      bind:value={content}
      class="w-full h-full bg-transparent border-none outline-none resize-none font-mono"
      placeholder="Quick capture..."
      autofocus
    ></textarea>
  </div>
  <div data-layout="capture-footer" class="font-mono text-xs flex justify-between items-center px-2">
    <VimModeIndicator />
    <div class="flex gap-2">
      <button onclick={handleCancel} class="px-2 py-1 text-[var(--cm-comment)]">Cancel</button>
      <button onclick={handleSubmit} class="px-2 py-1 bg-[var(--cm-accent)] text-[var(--cm-accent-foreground)]" disabled={saving}>
        {saving ? 'Saving...' : 'Save'}
      </button>
    </div>
  </div>
</div>
