import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
import { languages } from '@codemirror/language-data';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { keymap, EditorView, drawSelection } from '@codemirror/view';
import { vim } from '@replit/codemirror-vim';
import { type Extension } from '@codemirror/state';
import { tokyoNightTheme } from './codemirror-theme';

/**
 * Create the base extensions that don't change
 */
export function createBaseExtensions(): Extension[] {
	return [
		// Markdown support with GFM and code block highlighting
		markdown({
			base: markdownLanguage,
			codeLanguages: languages
		}),

		// History for undo/redo
		history(),

		// Keymaps
		keymap.of([...defaultKeymap, ...historyKeymap]),

		// Line wrapping for prose
		EditorView.lineWrapping,

		// Custom selection rendering (required for vim mode visual selection)
		// The vim plugin hides native ::selection, so we need drawSelection()
		// to render selection backgrounds via cm-selectionLayer
		drawSelection(),

		// Tokyo Night theme (automatically switches via CSS variables)
		tokyoNightTheme
	];
}

/**
 * Create the full extension set (always includes vim mode)
 */
export function createExtensions(): Extension[] {
	return [
		// Vim mode (always enabled)
		vim(),

		// Base extensions (includes theme)
		...createBaseExtensions()
	];
}
