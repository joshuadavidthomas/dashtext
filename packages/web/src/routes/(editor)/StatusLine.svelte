<script lang="ts">
  import * as AppBar from '@dashtext/lib/appbar';
  import { getEditorContext, VimModeIndicator } from '@dashtext/lib/editor';

  const editor = getEditorContext();

  function formatScroll(percent: number): string {
    if (percent === 0) return 'Top';
    if (percent >= 100) return 'Bot';
    return `${percent}%`;
  }
</script>

<AppBar.Root as="footer" data-layout="footer-bar" class="bg-[var(--cm-statusline-bg)] font-mono text-xs">
  <AppBar.Section>
    <VimModeIndicator />
  </AppBar.Section>

  <AppBar.Section class="text-[var(--cm-statusline-fg)]">
    <div class="gap-2 px-3 flex items-center py-0.5">
      <span>{editor.wordCount}w</span>
      <span>{editor.charCount}c</span>
    </div>

    <div class="px-2 py-0.5 bg-[var(--cm-statusline-section-bg)] text-[var(--cm-foreground)]">
      {formatScroll(editor.scrollPercent)}
    </div>

    <div class="px-2 py-0.5 bg-[var(--cm-accent)] font-semibold text-[var(--cm-accent-foreground)]">
      {editor.cursorLine}:{editor.cursorCol}
    </div>
  </AppBar.Section>
</AppBar.Root>
