import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags } from '@lezer/highlight';
import type { Extension } from '@codemirror/state';

/**
 * CodeMirror editor theme using CSS custom properties
 * Colors are defined in src/lib/styles/theme.css and automatically
 * switch between Tokyo Night Moon (dark) and Tokyo Night Day (light)
 */
export const tokyoNightEditorTheme = EditorView.theme({
	'&': {
		backgroundColor: 'var(--cm-background)',
		color: 'var(--cm-foreground)',
		height: '100%',
		fontSize: '16px',
		fontFamily: '"JetBrains Mono", monospace'
	},
	'.cm-scroller': {
		fontFamily: '"JetBrains Mono", monospace',
		lineHeight: '1.6',
		padding: '1rem'
	},
	'.cm-content': {
		caretColor: 'var(--cm-cursor)'
	},
	'.cm-cursor, .cm-dropCursor': {
		borderLeftColor: 'var(--cm-cursor)',
		borderLeftWidth: '2px'
	},
	// Selection background for drawSelection() layer
	// Override all CodeMirror default variants (light/dark, focused/unfocused)
	'.cm-selectionBackground': {
		backgroundColor: 'var(--cm-selection)'
	},
	'&.cm-focused .cm-selectionBackground': {
		backgroundColor: 'var(--cm-selection)'
	},
	'& .cm-selectionLayer .cm-selectionBackground': {
		backgroundColor: 'var(--cm-selection)'
	},
	'&.cm-focused .cm-selectionLayer .cm-selectionBackground': {
		backgroundColor: 'var(--cm-selection)'
	},
	// Match CodeMirror's highest specificity selectors
	'&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground': {
		backgroundColor: 'var(--cm-selection)'
	},
	'& > .cm-scroller > .cm-selectionLayer .cm-selectionBackground': {
		backgroundColor: 'var(--cm-selection)'
	},
	// Fallback for native selection (when not using drawSelection)
	'.cm-content ::selection': {
		backgroundColor: 'var(--cm-selection)'
	},
	'.cm-activeLine': {
		backgroundColor: 'var(--cm-active-line)'
	},
	'.cm-activeLineGutter': {
		backgroundColor: 'var(--cm-active-line)'
	},
	'.cm-gutters': {
		backgroundColor: 'var(--cm-gutter)',
		color: 'var(--cm-gutter-foreground)',
		border: 'none',
		display: 'none' /* Hidden for minimal look */
	},
	'.cm-lineNumbers .cm-gutterElement': {
		color: 'var(--cm-line-number)',
		padding: '0 8px'
	},
	'.cm-foldGutter .cm-gutterElement': {
		color: 'var(--cm-line-number)'
	},
	'&.cm-focused': {
		outline: 'none'
	},
	'.cm-matchingBracket, .cm-nonmatchingBracket': {
		backgroundColor: 'var(--cm-matching-bracket)',
		outline: '1px solid var(--cm-accent)'
	},
	'.cm-searchMatch': {
		backgroundColor: 'var(--cm-selection)'
	},
	'.cm-searchMatch.cm-searchMatch-selected': {
		backgroundColor: 'var(--cm-accent)'
	},
	'.cm-selectionMatch': {
		backgroundColor: 'var(--cm-selection)'
	},
	'.cm-panels': {
		backgroundColor: 'var(--cm-background-dark)',
		color: 'var(--cm-foreground)'
	},
	'.cm-panels.cm-panels-top': {
		borderBottom: '1px solid var(--cm-gutter)'
	},
	'.cm-panels.cm-panels-bottom': {
		borderTop: '1px solid var(--cm-gutter)'
	},
	'.cm-tooltip': {
		backgroundColor: 'var(--cm-background-dark)',
		color: 'var(--cm-foreground)',
		border: '1px solid var(--cm-gutter)'
	},
	'.cm-tooltip-autocomplete': {
		'& > ul > li[aria-selected]': {
			backgroundColor: 'var(--cm-selection)',
			color: 'var(--cm-foreground)'
		}
	},

	/* Vim mode cursor styles */
	'.cm-vimMode .cm-line': {
		caretColor: 'transparent'
	},
	'.cm-fat-cursor': {
		position: 'absolute',
		background: 'var(--cm-cursor)',
		border: 'none',
		whiteSpace: 'pre'
	},
	'&:not(.cm-focused) .cm-fat-cursor': {
		background: 'none',
		outline: '1px solid var(--cm-cursor)'
	},
	'.cm-vim-panel': {
		backgroundColor: 'var(--cm-statusline-bg)',
		color: 'var(--cm-statusline-fg)',
		padding: '2px 8px',
		fontFamily: '"JetBrains Mono", monospace',
		fontSize: '14px'
	},
	'.cm-vim-panel input': {
		backgroundColor: 'transparent',
		color: 'var(--cm-foreground)',
		border: 'none',
		outline: 'none',
		fontFamily: '"JetBrains Mono", monospace',
		fontSize: '14px'
	}
});

/**
 * Syntax highlighting using CSS custom properties
 */
export const tokyoNightHighlightStyle = HighlightStyle.define([
	// Comments
	{ tag: tags.comment, color: 'var(--cm-comment)', fontStyle: 'italic' },
	{ tag: tags.lineComment, color: 'var(--cm-comment)', fontStyle: 'italic' },
	{ tag: tags.blockComment, color: 'var(--cm-comment)', fontStyle: 'italic' },

	// Strings
	{ tag: tags.string, color: 'var(--cm-string)' },
	{ tag: tags.special(tags.string), color: 'var(--cm-string)' },
	{ tag: tags.docString, color: 'var(--cm-string)' },

	// Numbers
	{ tag: tags.number, color: 'var(--cm-number)' },
	{ tag: tags.integer, color: 'var(--cm-number)' },
	{ tag: tags.float, color: 'var(--cm-number)' },

	// Keywords
	{ tag: tags.keyword, color: 'var(--cm-keyword)' },
	{ tag: tags.modifier, color: 'var(--cm-keyword)' },
	{ tag: tags.operatorKeyword, color: 'var(--cm-keyword)' },
	{ tag: tags.controlKeyword, color: 'var(--cm-keyword)' },
	{ tag: tags.definitionKeyword, color: 'var(--cm-keyword)' },

	// Operators and punctuation
	{ tag: tags.operator, color: 'var(--cm-operator)' },
	{ tag: tags.punctuation, color: 'var(--cm-punctuation)' },
	{ tag: tags.bracket, color: 'var(--cm-punctuation)' },
	{ tag: tags.paren, color: 'var(--cm-punctuation)' },
	{ tag: tags.brace, color: 'var(--cm-punctuation)' },
	{ tag: tags.squareBracket, color: 'var(--cm-punctuation)' },
	{ tag: tags.angleBracket, color: 'var(--cm-punctuation)' },

	// Variables and identifiers
	{ tag: tags.variableName, color: 'var(--cm-variable)' },
	{ tag: tags.definition(tags.variableName), color: 'var(--cm-variable)' },
	{ tag: tags.local(tags.variableName), color: 'var(--cm-variable)' },
	{ tag: tags.special(tags.variableName), color: 'var(--cm-variable)' },

	// Functions
	{ tag: tags.function(tags.variableName), color: 'var(--cm-function)' },
	{ tag: tags.definition(tags.function(tags.variableName)), color: 'var(--cm-function)' },

	// Types and classes
	{ tag: tags.typeName, color: 'var(--cm-type)' },
	{ tag: tags.className, color: 'var(--cm-class)' },
	{ tag: tags.namespace, color: 'var(--cm-type)' },

	// Properties
	{ tag: tags.propertyName, color: 'var(--cm-property)' },
	{ tag: tags.definition(tags.propertyName), color: 'var(--cm-property)' },

	// Constants
	{ tag: tags.constant(tags.variableName), color: 'var(--cm-constant)' },
	{ tag: tags.bool, color: 'var(--cm-constant)' },
	{ tag: tags.null, color: 'var(--cm-constant)' },

	// HTML/XML tags and attributes
	{ tag: tags.tagName, color: 'var(--cm-tag)' },
	{ tag: tags.attributeName, color: 'var(--cm-attribute)' },
	{ tag: tags.attributeValue, color: 'var(--cm-string)' },

	// Markdown specific
	{ tag: tags.heading, color: 'var(--cm-heading)', fontWeight: 'bold' },
	{ tag: tags.heading1, color: 'var(--cm-heading)', fontWeight: 'bold', fontSize: '1.5em' },
	{ tag: tags.heading2, color: 'var(--cm-heading)', fontWeight: 'bold', fontSize: '1.3em' },
	{ tag: tags.heading3, color: 'var(--cm-heading)', fontWeight: 'bold', fontSize: '1.15em' },
	{ tag: tags.heading4, color: 'var(--cm-heading)', fontWeight: 'bold' },
	{ tag: tags.heading5, color: 'var(--cm-heading)', fontWeight: 'bold' },
	{ tag: tags.heading6, color: 'var(--cm-heading)', fontWeight: 'bold' },
	{ tag: tags.link, color: 'var(--cm-link)', textDecoration: 'underline' },
	{ tag: tags.url, color: 'var(--cm-url)' },
	{ tag: tags.emphasis, color: 'var(--cm-emphasis)', fontStyle: 'italic' },
	{ tag: tags.strong, color: 'var(--cm-strong)', fontWeight: 'bold' },
	{ tag: tags.monospace, color: 'var(--cm-code)', fontFamily: '"JetBrains Mono", monospace' },
	{ tag: tags.quote, color: 'var(--cm-quote)', fontStyle: 'italic' },
	{ tag: tags.list, color: 'var(--cm-list)' },

	// Misc
	{ tag: tags.invalid, color: 'var(--cm-error)' },
	{ tag: tags.meta, color: 'var(--cm-comment)' },
	{ tag: tags.processingInstruction, color: 'var(--cm-comment)' }
]);

/**
 * Complete Tokyo Night theme extension for CodeMirror
 * Combines editor theme and syntax highlighting
 */
export const tokyoNightTheme: Extension = [
	tokyoNightEditorTheme,
	syntaxHighlighting(tokyoNightHighlightStyle)
];
