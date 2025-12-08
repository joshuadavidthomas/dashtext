<script lang="ts">
  import { EditorView } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { getCM } from '@replit/codemirror-vim';
  import { createCaptureExtensions, setActiveCallbacks } from './extensions';
  import { editorState, type VimModeType } from '$lib/components/editor/context.svelte';

  interface Props {
    onSubmitClose: () => void;
    onSubmitContinue: () => void;
    onClose: () => void;
    getContent?: (getter: () => string) => void;
    clearEditor?: (clearer: () => void) => void;
  }

  let {
    onSubmitClose,
    onSubmitContinue,
    onClose,
    getContent,
    clearEditor
  }: Props = $props();

  let view: EditorView | null = null;
  let content = $state('');

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

  // Vim mode change handler reference for cleanup
  let vimModeChangeHandler: ((event: { mode: string; subMode?: string }) => void) | null = null;

  // Setup vim mode change listener
  function setupVimModeListener(v: EditorView) {
    const cm = getCM(v);
    if (!cm) return;

    vimModeChangeHandler = (event: { mode: string; subMode?: string }) => {
      queueMicrotask(() => editorState.setVimMode(mapVimMode(event.mode, event.subMode)));
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

  // Expose content getter to parent
  $effect(() => {
    if (getContent) {
      getContent(() => content);
    }
  });

  // Expose clear function to parent
  $effect(() => {
    if (clearEditor) {
      clearEditor(() => {
        if (view) {
          view.dispatch({
            changes: {
              from: 0,
              to: view.state.doc.length,
              insert: ''
            }
          });
          content = '';
          view.focus();
        }
      });
    }
  });

  // Action for editor initialization
  function initEditor(container: HTMLDivElement) {
    // Validation is handled by parent component (+page.svelte)
    const callbacks = {
      onSubmitClose,
      onSubmitContinue,
      onClose
    };

    const state = EditorState.create({
      doc: '',
      extensions: [
        ...createCaptureExtensions(callbacks),
        // Update listener to track content
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            content = update.state.doc.toString();
          }
        }),
        // Focus/blur handlers - defer state updates to avoid mutation during render
        EditorView.domEventHandlers({
          focus: () => {
            queueMicrotask(() => editorState.setFocused(true));
          },
          blur: () => {
            queueMicrotask(() => editorState.setFocused(false));
          }
        })
      ]
    });

    view = new EditorView({ state, parent: container });
    view.focus();
    queueMicrotask(() => {
      editorState.setFocused(true);
      editorState.setVimMode('normal');
    });
    setupVimModeListener(view);

    return {
      destroy() {
        if (view) {
          cleanupVimModeListener(view);
          setActiveCallbacks(null);
          view.destroy();
          view = null;
        }
      }
    };
  }
</script>

<div use:initEditor class="h-full w-full [&_.cm-editor]:h-full" role="textbox" aria-label="Quick capture editor"></div>
