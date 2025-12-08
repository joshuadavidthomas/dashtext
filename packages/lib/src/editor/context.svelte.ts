import { createContext } from 'svelte';

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
  }
}

export const [getEditorContext, setEditorContext] = createContext<EditorState>();

export function createEditorContext(): EditorState {
  const state = new EditorState();
  setEditorContext(state);
  return state;
}
