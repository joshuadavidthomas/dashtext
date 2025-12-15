import type { DraftsAPI, Draft } from '../stores';

/**
 * Platform capabilities interface - allows shared components to adapt
 * to platform features without importing platform-specific code.
 */
export interface PlatformCapabilities {
  /** Platform identifier for conditional rendering */
  platform: 'desktop' | 'web';

  /** Subscribe to app focus changes, returns cleanup function */
  onFocusChange(callback: () => void): () => void;

  /** Window control capabilities (null if not supported) */
  window: {
    minimize(): Promise<void>;
    maximize(): Promise<void>;
    close(): Promise<void>;
  } | null;

  /** Quick capture capability (null if not supported) */
  quickCapture: {
    open(): Promise<void>;
  } | null;

  /** Settings window capability (null if not supported, e.g. web uses dialog instead) */
  settings: {
    open(): Promise<void>;
  } | null;

  /** Drafts API adapter - platform-specific storage implementation */
  draftsAPI: DraftsAPI;

  /** Refresh drafts list from storage */
  refreshDrafts(): Promise<Draft[]>;

  /** Navigation capabilities */
  replaceUrl: (url: string) => void;
  navigateTo: (url: string) => Promise<void>;
}
