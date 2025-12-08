<script lang="ts">
	import { editorState, type VimModeType } from './context.svelte';

	const editor = editorState;

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

	const modeDisplay = $derived(getModeDisplay(editor.vimMode));
</script>

<div class="w-fit px-2 py-0.5 font-semibold text-[var(--cm-mode-fg)] {modeDisplay.class}">
	{modeDisplay.text}
</div>
