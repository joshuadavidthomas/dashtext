<script lang="ts">
  import { drafts } from '$lib/api';
  import { CaptureEditor } from '$lib/components/capture';
  import { VimModeIndicator } from '@dashtext/lib/editor';
  import { getCurrentWindow } from '@tauri-apps/api/window';

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
    await getCurrentWindow().close();
  }

  async function handleSubmitClose() {
    const content = getContent();
    if (!content.trim() || saving) return;

    saving = true;
    try {
      const draft = await drafts.create();
      await drafts.save(draft.uuid, content.trim());
      await closeCapture();
    } finally {
      saving = false;
    }
  }

  async function handleSubmitContinue() {
    const content = getContent();
    if (!content.trim() || saving) return;

     saving = true;
    try {
      const draft = await drafts.create();
      await drafts.save(draft.uuid, content.trim());
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
