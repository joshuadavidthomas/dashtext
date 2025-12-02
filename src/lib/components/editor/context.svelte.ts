import { getContext, setContext } from 'svelte';

const EDITOR_CONTEXT_KEY = Symbol('editor-context');

export type VimModeType =
	| 'normal'
	| 'insert'
	| 'visual'
	| 'visual-line'
	| 'visual-block'
	| 'replace'
	| 'command'
	| null;

/**
 * Editor state class using Svelte 5 runes
 * Manages content, cursor position, and vim mode state
 */
export class EditorState {
	content = $state('');

	// Vim mode state (null when vim is disabled)
	vimMode = $state<VimModeType>(null);

	// Editor (CodeMirror) focus state
	isFocused = $state(false);
	
	// Container wrapper focus state (when tabbed to but not editing)
	containerFocused = $state(false);
	
	// Tracks if user exited via Escape (shows border indicator)
	escapedOut = $state(false);
	
	// Show border when container is focused OR just escaped
	readonly showBorder = $derived(this.containerFocused || this.escapedOut);

	// Cursor position
	cursorLine = $state(1);
	cursorCol = $state(1);
	totalLines = $state(1);

	// Cursor position as percentage through document (0-100)
	readonly scrollPercent = $derived(
		this.totalLines <= 1 ? 0 : Math.round(((this.cursorLine - 1) / (this.totalLines - 1)) * 100)
	);

	readonly wordCount = $derived(
		this.content.trim() === '' ? 0 : this.content.trim().split(/\s+/).length
	);

	readonly charCount = $derived(this.content.length);

	setContent(value: string) {
		this.content = value;
	}

	setVimMode(mode: VimModeType) {
		this.vimMode = mode;
	}

	setCursorPosition(line: number, col: number) {
		this.cursorLine = line;
		this.cursorCol = col;
	}

	setTotalLines(lines: number) {
		this.totalLines = lines;
	}

	setFocused(focused: boolean) {
		this.isFocused = focused;
		// Clear escapedOut and containerFocused when editor regains focus
		if (focused) {
			this.escapedOut = false;
			this.containerFocused = false;
		}
	}

	setContainerFocused(focused: boolean) {
		this.containerFocused = focused;
	}

	setEscapedOut(escaped: boolean) {
		this.escapedOut = escaped;
	}
}

/**
 * Create and set the editor context
 * Call this in the parent component that owns the editor
 */
export function createEditorContext(): EditorState {
	const state = new EditorState();
	setContext(EDITOR_CONTEXT_KEY, state);
	return state;
}

/**
 * Get the editor context
 * Call this in child components that need access to editor state
 */
export function getEditorContext(): EditorState {
	return getContext<EditorState>(EDITOR_CONTEXT_KEY);
}
