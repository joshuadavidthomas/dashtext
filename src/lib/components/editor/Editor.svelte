<script lang="ts">
	import { EditorView } from '@codemirror/view';
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

	// Update scroll percentage
	function updateScrollPercent(view: EditorView) {
		const scrollDOM = view.scrollDOM;
		const scrollTop = scrollDOM.scrollTop;
		const scrollHeight = scrollDOM.scrollHeight - scrollDOM.clientHeight;
		const percent = scrollHeight > 0 ? Math.round((scrollTop / scrollHeight) * 100) : 0;
		editorState.setScrollPercent(percent);
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

	// Action for editor initialization - runs once on mount
	function initEditor(container: HTMLDivElement) {
		const state = EditorState.create({
			doc: '',
			extensions: [
				...createExtensions({ vimMode }),
				// Update listener to sync content and cursor to context
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						editorState.setContent(update.state.doc.toString());
					}
					if (update.selectionSet || update.docChanged) {
						updateCursorPosition(update.view);
					}
				}),
				// Scroll listener
				EditorView.domEventHandlers({
					scroll: (_, view) => {
						updateScrollPercent(view);
						return false;
					}
				})
			]
		});

		const view = new EditorView({ state, parent: container });
		view.focus();

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

		return {
			destroy() {
				cleanupVimModeListener(view);
				view.destroy();
			}
		};
	}
</script>

<div use:initEditor class="editor-container"></div>

<style>
	.editor-container {
		width: 100%;
		height: 100%;
	}

	.editor-container :global(.cm-editor) {
		height: 100%;
	}
</style>
