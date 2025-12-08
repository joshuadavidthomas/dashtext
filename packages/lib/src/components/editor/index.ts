export { default as Editor } from './Editor.svelte';
export { default as VimModeIndicator } from './VimModeIndicator.svelte';
export { createEditorContext, getEditorContext, EditorState, type VimModeType } from './context.svelte';
export { createExtensions, createBaseExtensions } from './extensions';
export { tokyoNightTheme, tokyoNightEditorTheme, tokyoNightHighlightStyle } from './codemirror-theme';
