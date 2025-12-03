<script lang="ts">
	import { getEditorContext, type VimModeType } from '$lib/components/editor';
	import * as AppBar from '$lib/components/layout/app-bar';

	const editor = getEditorContext();

	// Get the display text and CSS class for vim mode
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

	// Format scroll percentage display
	function formatScroll(percent: number): string {
		if (percent === 0) return 'Top';
		if (percent >= 100) return 'Bot';
		return `${percent}%`;
	}

	const modeDisplay = $derived(getModeDisplay(editor.vimMode));
</script>

<footer data-layout="footer-bar">
	<AppBar.Root class="bg-[var(--cm-statusline-bg)] font-mono text-xs">
		<AppBar.Section>
			{#if editor.vimMode}
				<AppBar.Item class="font-semibold text-[var(--cm-mode-fg)] {modeDisplay.class}">
					{modeDisplay.text}
				</AppBar.Item>
			{/if}
		</AppBar.Section>

		<AppBar.Section class="text-[var(--cm-statusline-fg)]">
			<AppBar.Item class="gap-2 px-3">
				<span>{editor.wordCount}w</span>
				<span>{editor.charCount}c</span>
			</AppBar.Item>

			<AppBar.Item class="bg-[var(--cm-statusline-section-bg)] text-[var(--cm-foreground)]">
				{formatScroll(editor.scrollPercent)}
			</AppBar.Item>

			<AppBar.Item class="bg-[var(--cm-accent)] font-semibold text-[var(--cm-accent-foreground)]">
				{editor.cursorLine}:{editor.cursorCol}
			</AppBar.Item>
		</AppBar.Section>
	</AppBar.Root>
</footer>
