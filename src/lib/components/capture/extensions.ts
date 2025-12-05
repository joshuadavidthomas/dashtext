import { type Extension } from '@codemirror/state';
import { Vim } from '@replit/codemirror-vim';
import { createBaseExtensions } from '$lib/components/editor/extensions';
import { vim } from '@replit/codemirror-vim';

export interface CaptureCallbacks {
  onSubmitClose: () => void;
  onSubmitContinue: () => void;
  onClose: () => void;
}

// Track if we need to set up the vim commands (only once globally)
// Note: Vim commands are global and can't be unregistered, so we register once
let vimCommandsRegistered = false;

// Store callbacks globally so vim commands can access them
// This is necessary because Vim.defineEx doesn't have a way to pass context
// Safe because we enforce single capture window in openQuickCapture()
let activeCallbacks: CaptureCallbacks | null = null;

export function setActiveCallbacks(callbacks: CaptureCallbacks | null) {
  activeCallbacks = callbacks;
}

function registerVimCommands() {
  if (vimCommandsRegistered) return;

  // Define custom Ex commands for capture actions
  Vim.defineEx('captureclose', '', () => {
    activeCallbacks?.onSubmitClose();
  });

  Vim.defineEx('capturecontinue', '', () => {
    activeCallbacks?.onSubmitContinue();
  });

  Vim.defineEx('capturecancel', '', () => {
    activeCallbacks?.onClose();
  });

  // Map Enter in normal mode to submit and close
  Vim.map('<CR>', ':captureclose<CR>', 'normal');

  // Map Ctrl+Enter in normal mode to submit and continue
  Vim.map('<C-CR>', ':capturecontinue<CR>', 'normal');

  vimCommandsRegistered = true;
}

/**
 * Create extensions for the capture editor
 * Includes vim mode with custom keybindings for capture actions
 */
export function createCaptureExtensions(callbacks: CaptureCallbacks): Extension[] {
  // Register vim commands (idempotent)
  registerVimCommands();

  // Set active callbacks for this editor instance
  setActiveCallbacks(callbacks);

  return [
    // Vim mode
    vim(),

    // Base extensions (markdown, history, theme, etc.)
    // Note: Double-escape to close is handled at document level in +page.svelte
    ...createBaseExtensions()
  ];
}
