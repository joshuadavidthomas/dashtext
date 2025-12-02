<script lang="ts">
	import { EditorView, keymap } from '@codemirror/view';
	import { EditorState } from '@codemirror/state';
	import { vim, getCM } from '@replit/codemirror-vim';
	import { watch } from 'runed';
	import { createExtensions, vimCompartment } from './extensions';
	import { getEditorContext, type VimModeType } from './context.svelte';

	interface Props {
		vimMode?: boolean;
	}

	let { vimMode = false }: Props = $props();

	const editorState = getEditorContext();

	// Reference to the container element for focus management
	let containerEl: HTMLDivElement | null = null;
	
	// Reference to the CodeMirror view for re-focusing
	let editorView: EditorView | null = null;

	// Ghost cursor element (injected into CM scroller)
	let ghostCursorEl: HTMLDivElement | null = null;

	// Create and inject ghost cursor element into CodeMirror
	function createGhostCursor(view: EditorView) {
		ghostCursorEl = document.createElement('div');
		ghostCursorEl.className = 'ghost-cursor';
		ghostCursorEl.style.cssText = `
			position: absolute;
			pointer-events: none;
			width: 2px;
			background-color: var(--cm-cursor);
			z-index: 10;
			display: none;
		`;
		view.scrollDOM.appendChild(ghostCursorEl);
	}

	// Capture cursor coordinates and show ghost cursor
	function showGhostCursor(view: EditorView) {
		if (!ghostCursorEl) return;
		
		const pos = view.state.selection.main.head;
		const coords = view.coordsAtPos(pos);
		if (!coords) return;

		const scrollerRect = view.scrollDOM.getBoundingClientRect();
		
		ghostCursorEl.style.top = `${coords.top - scrollerRect.top + view.scrollDOM.scrollTop}px`;
		ghostCursorEl.style.left = `${coords.left - scrollerRect.left + view.scrollDOM.scrollLeft}px`;
		ghostCursorEl.style.height = `${coords.bottom - coords.top}px`;
		ghostCursorEl.style.display = 'block';
	}

	// Hide ghost cursor
	function hideGhostCursor() {
		if (ghostCursorEl) {
			ghostCursorEl.style.display = 'none';
		}
	}

	// Map vim mode strings to our type
	function mapVimMode(mode: string, subMode?: string): VimModeType {
		if (mode === 'visual') {
			if (subMode === 'linewise') return 'visual-line';
			if (subMode === 'blockwise') return 'visual-block';
			return 'visual';
		}
		switch (mode) {
			case 'normal':
				return 'normal';
			case 'insert':
				return 'insert';
			case 'replace':
				return 'replace';
			default:
				return 'normal';
		}
	}

	// Update cursor position from editor state
	function updateCursorPosition(view: EditorView) {
		const pos = view.state.selection.main.head;
		const line = view.state.doc.lineAt(pos);
		editorState.setCursorPosition(line.number, pos - line.from + 1);
		editorState.setTotalLines(view.state.doc.lines);
	}

	// Vim mode change handler reference for cleanup
	let vimModeChangeHandler: ((event: { mode: string }) => void) | null = null;

	// Setup vim mode change listener
	function setupVimModeListener(view: EditorView) {
		const cm = getCM(view);
		if (!cm) return;

		vimModeChangeHandler = (event: { mode: string; subMode?: string }) => {
			editorState.setVimMode(mapVimMode(event.mode, event.subMode));
		};

		cm.on('vim-mode-change', vimModeChangeHandler);
	}

	// Cleanup vim mode listener
	function cleanupVimModeListener(view: EditorView) {
		if (!vimModeChangeHandler) return;
		const cm = getCM(view);
		if (!cm) return;

		cm.off('vim-mode-change', vimModeChangeHandler);
		vimModeChangeHandler = null;
	}

	// Create Escape key handler that exits editor to container
	// In vim mode: only exit when in normal mode (Escape in insert goes to normal first)
	// In non-vim mode: always exit on Escape
	function createEscapeHandler(view: EditorView): boolean {
		// In vim mode, only exit if we're already in normal mode
		if (vimMode && editorState.vimMode !== 'normal') {
			return false; // Let vim handle the Escape
		}
		showGhostCursor(view);
		editorState.setEscapedOut(true);
		containerEl?.focus();
		return true;
	}

	// Action for editor initialization - runs once on mount
	function initEditor(container: HTMLDivElement) {
		const state = EditorState.create({
			doc: '',
			extensions: [
				...createExtensions({ vimMode }),
				// Escape key handler for exiting editor focus
				keymap.of([
					{
						key: 'Escape',
						run: createEscapeHandler
					}
				]),
				// Focus change listener
				EditorView.domEventHandlers({
					focus: () => {
						hideGhostCursor();
						editorState.setFocused(true);
					},
					blur: () => {
						editorState.setFocused(false);
					}
				}),
				// Update listener to sync content and cursor to context
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						editorState.setContent(update.state.doc.toString());
					}
					if (update.selectionSet || update.docChanged) {
						updateCursorPosition(update.view);
					}
				})
			]
		});

		const view = new EditorView({ state, parent: container });
		editorView = view;
		createGhostCursor(view);
		view.focus();
		editorState.setFocused(true);

		// Set initial vim mode state and setup listener
		if (vimMode) {
			editorState.setVimMode('normal');
			setupVimModeListener(view);
		}

		// Watch vim mode toggle changes
		watch(
			() => vimMode,
			(enabled) => {
				// Cleanup old listener before reconfiguring
				cleanupVimModeListener(view);

				view.dispatch({
					effects: vimCompartment.reconfigure(enabled ? vim() : [])
				});

				// Update vim mode state in context and setup new listener
				if (enabled) {
					editorState.setVimMode('normal');
					// Need to wait for the vim extension to initialize
					requestAnimationFrame(() => setupVimModeListener(view));
				} else {
					editorState.setVimMode(null);
				}
			},
			{ lazy: true }
		);

		// Listen for Tab key at document level to clear escaped state
		function handleDocumentKeydown(e: KeyboardEvent) {
			if (e.key === 'Tab' && editorState.escapedOut) {
				editorState.setEscapedOut(false);
			}
		}
		document.addEventListener('keydown', handleDocumentKeydown);

		// Remove CM content from tab order - only focusable via click/Enter/typing
		view.contentDOM.setAttribute('tabindex', '-1');

		return {
			destroy() {
				cleanupVimModeListener(view);
				document.removeEventListener('keydown', handleDocumentKeydown);
				ghostCursorEl?.remove();
				ghostCursorEl = null;
				view.destroy();
				editorView = null;
			}
		};
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex a11y_no_noninteractive_element_interactions -->
<div
	bind:this={containerEl}
	use:initEditor
	class="editor-container"
	class:show-border={editorState.showBorder}
	tabindex="0"
	onfocus={() => {
		editorState.setContainerFocused(true);
		// Show ghost cursor when tabbing back to container
		if (editorView) {
			showGhostCursor(editorView);
		}
	}}
	onblur={() => editorState.setContainerFocused(false)}
	onmousedown={() => editorView?.focus()}
	onkeydown={(e) => {
		// Enter or any printable character focuses editor
		if (e.key === 'Enter') {
			e.preventDefault();
			editorView?.focus();
		} else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
			// Printable character - focus editor and let it handle the key
			editorView?.focus();
		}
	}}
	role="application"
	aria-label="Text editor"
></div>

<style>
	.editor-container {
		position: relative;
		width: 100%;
		height: 100%;
		border: 2px solid transparent;
		transition: border-color 0.15s ease;
		outline: none;
	}

	.editor-container.show-border {
		border-color: var(--cm-accent);
	}

	.editor-container :global(.cm-editor) {
		height: 100%;
	}

</style>
