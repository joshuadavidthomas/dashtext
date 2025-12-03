<script lang="ts">
	import { getEditorContext, type VimModeType } from '$lib/components/editor';

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

<footer
	data-layout="footer-bar"
	class="flex cursor-default select-none items-stretch justify-between bg-[var(--cm-statusline-bg)] p-1 font-mono text-xs"
>
	<!-- Left section: Mode indicator + VIM toggle -->
	<div class="flex items-stretch">
		<!-- Vim mode indicator (hidden when vim disabled) -->
		{#if editor.vimMode}
			<div
				class="flex items-center px-2 font-semibold text-[var(--cm-mode-fg)] {modeDisplay.class}"
			>
				{modeDisplay.text}
			</div>
		{/if}
	</div>

	<!-- Right section: Stats and position -->
	<div class="flex items-stretch text-[var(--cm-statusline-fg)]">
		<!-- Word and char counts -->
		<div class="flex items-center gap-2 px-3">
			<span>{editor.wordCount}w</span>
			<span>{editor.charCount}c</span>
		</div>

		<!-- Scroll percentage -->
		<div class="flex items-center bg-[var(--cm-statusline-section-bg)] px-2 text-[var(--cm-foreground)]">
			{formatScroll(editor.scrollPercent)}
		</div>

		<!-- Cursor position -->
		<div class="flex items-center bg-[var(--cm-accent)] px-2 font-semibold text-[var(--cm-accent-foreground)]">
			{editor.cursorLine}:{editor.cursorCol}
		</div>
	</div>
</footer>
