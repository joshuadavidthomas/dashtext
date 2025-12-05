<script lang="ts">
  import { createDraft, saveDraft } from '$lib/api';
  import { CaptureEditor } from '$lib/components/capture';
  import { VimModeIndicator } from '$lib/components/editor';
  import { isTauri } from '$lib/platform';
  import { goto } from '$app/navigation';

  const inTauri = isTauri();

  let saving = $state(false);
  let contentGetter: (() => string) | null = $state(null);
  let editorClearer: (() => void) | null = $state(null);

  function getContent(): string {
    return contentGetter?.() ?? '';
  }

  function clearEditor() {
    editorClearer?.();
  }

  async function closeCapture() {
    if (inTauri) {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    } else {
      // On web, navigate back to main editor
      goto('/drafts/new');
    }
  }

  async function handleSubmitClose() {
    const content = getContent();
    if (!content.trim() || saving) return;

    saving = true;
    try {
      const draft = await createDraft();
      await saveDraft(draft.id, content.trim());
      if (inTauri) {
        await closeCapture();
      } else {
        // On web, navigate to the newly created draft
        goto(`/drafts/${draft.id}`);
      }
    } finally {
      saving = false;
    }
  }

  async function handleSubmitContinue() {
    const content = getContent();
    if (!content.trim() || saving) return;

    saving = true;
    try {
      const draft = await createDraft();
      await saveDraft(draft.id, content.trim());
      clearEditor();
    } finally {
      saving = false;
    }
  }

  async function handleClose() {
    await closeCapture();
  }

  // Document-level double-escape handler (works even when editor isn't focused)
  $effect(() => {
    let lastEscape = 0;

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        const now = Date.now();
        if (now - lastEscape < 500) {
          handleClose();
        }
        lastEscape = now;
      }
    }

    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });
</script>

<div data-layout="capture-root">
  <div data-layout="capture-editor">
    <CaptureEditor
      onSubmitClose={handleSubmitClose}
      onSubmitContinue={handleSubmitContinue}
      onClose={handleClose}
      getContent={(getter: () => string) => (contentGetter = getter)}
      clearEditor={(clearer: () => void) => (editorClearer = clearer)}
    />
  </div>
  <div data-layout="capture-footer" class="font-mono text-xs">
    <VimModeIndicator />
  </div>
</div>
