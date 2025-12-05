<script lang="ts">
	import * as AppBar from '$lib/components/appbar';
	import { getEditorContext, type VimModeType } from '$lib/components/editor';
	import { VersionIndicator } from '$lib/components/updater';

	const editor = getEditorContext();

	function getModeDisplay(mode: VimModeType): { text: string; class: string } {
		switch (mode) {
			case 'normal':
				return { text: 'NORMAL', class: 'bg-[var(--cm-mode-normal-bg)]' };
			case 'insert':
				return { text: 'INSERT', class: 'bg-[var(--cm-mode-insert-bg)]' };
			case 'visual':
				return { text: 'VISUAL', class: 'bg-[var(--cm-mode-visual-bg)]' };
			case 'visual-line':
				return { text: 'V-LINE', class: 'bg-[var(--cm-mode-visual-bg)]' };
			case 'visual-block':
				return { text: 'V-BLOCK', class: 'bg-[var(--cm-mode-visual-bg)]' };
			case 'replace':
				return { text: 'REPLACE', class: 'bg-[var(--cm-mode-replace-bg)]' };
			case 'command':
				return { text: 'COMMAND', class: 'bg-[var(--cm-mode-command-bg)]' };
			default:
				return { text: '', class: '' };
		}
	}

	function formatScroll(percent: number): string {
		if (percent === 0) return 'Top';
		if (percent >= 100) return 'Bot';
		return `${percent}%`;
	}

	const modeDisplay = $derived(getModeDisplay(editor.vimMode));
</script>

<AppBar.Root as="footer" data-layout="footer-bar" class="bg-[var(--cm-statusline-bg)] font-mono text-xs">
	<AppBar.Section>
    <div class="px-2 py-0.5 font-semibold text-[var(--cm-mode-fg)] {modeDisplay.class}">
      {modeDisplay.text}
    </div>
		<VersionIndicator />
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
