<script lang="ts">
	import { EditorView } from '@codemirror/view';
	import { Annotation, EditorState } from '@codemirror/state';
	import { getCM } from '@replit/codemirror-vim';
	import { createExtensions } from './extensions';
	import { getEditorContext, type VimModeType } from './context.svelte';
	import { getDraftsState } from '../stores/drafts.svelte';

	const draftsState = getDraftsState();
	const editorState = getEditorContext();

	// Store editor view reference for external updates
	let view: EditorView | null = null;

	// Annotation to mark transactions that sync content from draft (not user edits)
	const syncFromDraft = Annotation.define<boolean>();

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
	function updateCursorPosition(v: EditorView) {
		const pos = v.state.selection.main.head;
		const line = v.state.doc.lineAt(pos);
		editorState.setCursorPosition(line.number, pos - line.from + 1);
		editorState.setTotalLines(v.state.doc.lines);
	}

	// Vim mode change handler reference for cleanup
	let vimModeChangeHandler: ((event: { mode: string }) => void) | null = null;

	// Setup vim mode change listener
	function setupVimModeListener(v: EditorView) {
		const cm = getCM(v);
		if (!cm) return;

		vimModeChangeHandler = (event: { mode: string; subMode?: string }) => {
			editorState.setVimMode(mapVimMode(event.mode, event.subMode));
		};

		cm.on('vim-mode-change', vimModeChangeHandler);
	}

	// Cleanup vim mode listener
	function cleanupVimModeListener(v: EditorView) {
		if (!vimModeChangeHandler) return;
		const cm = getCM(v);
		if (!cm) return;

		cm.off('vim-mode-change', vimModeChangeHandler);
		vimModeChangeHandler = null;
	}

	// Action for editor initialization - runs once on mount
	function initEditor(container: HTMLDivElement) {
		const state = EditorState.create({
			doc: draftsState.currentDraft?.content ?? '',
			extensions: [
				...createExtensions(),
				// DOM event handlers
				EditorView.domEventHandlers({
					keydown: (event, v) => {
						// Trap Tab in all modes except normal (allow focus navigation in normal)
						if (event.key === 'Tab' && editorState.vimMode !== 'normal') {
							event.preventDefault();
							if (editorState.vimMode === 'insert') {
								v.dispatch(v.state.replaceSelection('\t'));
							}
							return true;
						}
						return false;
					},
					focus: () => {
						editorState.setFocused(true);
					},
					blur: () => {
						editorState.setFocused(false);
					}
				}),
				// Update listener to sync content and cursor to context
				EditorView.updateListener.of((update) => {
					const isSyncFromDraft = update.transactions.some((tr) => tr.annotation(syncFromDraft));
					if (update.docChanged && !isSyncFromDraft) {
						const content = update.state.doc.toString();
						editorState.setContent(content);
						draftsState.updateContent(content);
					}
					if (update.selectionSet || update.docChanged) {
						updateCursorPosition(update.view);
					}
				})
			]
		});

		view = new EditorView({ state, parent: container });
		view.focus();
		editorState.setFocused(true);

		// Set initial vim mode state and setup listener
		editorState.setVimMode('normal');
		setupVimModeListener(view);

		return {
			destroy() {
				if (view) {
					cleanupVimModeListener(view);
					view.destroy();
					view = null;
				}
			}
		};
	}

	// Sync editor content when current draft changes
	$effect(() => {
		const draft = draftsState.currentDraft;
		if (view) {
			const newContent = draft?.content ?? '';
			const currentContent = view.state.doc.toString();
			if (currentContent !== newContent) {
				view.dispatch({
					changes: {
						from: 0,
						to: view.state.doc.length,
						insert: newContent
					},
					annotations: syncFromDraft.of(true)
				});
				editorState.setContent(newContent);
			}
		}
	});

	// Focus editor when navigating to different draft
	$effect(() => {
		draftsState.currentDraft; // track reference changes
		if (view) {
			view.focus();
		}
	});
</script>

<div use:initEditor class="editor-container" role="textbox" aria-label="Text editor"></div>

<style>
	.editor-container {
		width: 100%;
		height: 100%;
	}

	.editor-container :global(.cm-editor) {
		height: 100%;
	}
</style>
