import { type Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { Vim } from '@replit/codemirror-vim';
import { createBaseExtensions } from '$lib/components/editor/extensions';
import { vim } from '@replit/codemirror-vim';

export interface CaptureCallbacks {
  onSubmitClose: () => void;
  onSubmitContinue: () => void;
  onClose: () => void;
}

// Track if we need to set up the vim commands (only once globally)
let vimCommandsRegistered = false;

// Store callbacks globally so vim commands can access them
// This is necessary because Vim.defineEx doesn't have a way to pass context
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
    ...createBaseExtensions(),

    // Handle double-Escape to close (when already in normal mode)
    // We track the last Escape time and close if pressed twice quickly
    EditorView.domEventHandlers({
      keydown: (event) => {
        if (event.key === 'Escape') {
          // Check if we're in normal mode by looking at the vim state
          // If the previous key was also Escape within 500ms, close
          const now = Date.now();
          const lastEscape = (window as unknown as { __captureLastEscape?: number }).__captureLastEscape || 0;

          if (now - lastEscape < 500) {
            // Double escape - close the window
            callbacks.onClose();
            (window as unknown as { __captureLastEscape?: number }).__captureLastEscape = 0;
            return true;
          }

          (window as unknown as { __captureLastEscape?: number }).__captureLastEscape = now;
        }
        return false;
      }
    })
  ];
}
