import { getCurrentWindow } from '@tauri-apps/api/window';
import { replaceState, goto } from '$app/navigation';
import { drafts } from '$lib/api';
import type { PlatformCapabilities } from '@dashtext/lib/platform';

export const desktopPlatform: PlatformCapabilities = {
  platform: 'desktop',

  onFocusChange(callback) {
    let unlisten: (() => void) | null = null;

    getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused) callback();
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => unlisten?.();
  },

  window: {
    minimize: () => getCurrentWindow().minimize(),
    maximize: () => getCurrentWindow().toggleMaximize(),
    close: () => getCurrentWindow().close(),
  },

  quickCapture: {
    open: async () => {
      const m = await import('$lib/components/capture');
      await m.openQuickCapture();
    },
  },

  settings: {
    open: async () => {
      const m = await import('$lib/components/settings');
      await m.openSettings();
    },
  },

  replaceUrl: (url) => replaceState(url, {}),
  navigateTo: (url) => goto(url),

  draftsAPI: {
    createDraft: async () => drafts.create(),
    saveDraft: async (id, content) => drafts.save(id, content),
    deleteDraft: async (id) => drafts.delete(id),
    archiveDraft: async (id) => drafts.archive(id),
    unarchiveDraft: async (id) => drafts.unarchive(id),
    pinDraft: async (id) => drafts.pin(id),
    unpinDraft: async (id) => drafts.unpin(id),
    restoreDraft: async (id) => drafts.restore(id),
    hardDeleteDraft: async (id) => drafts.hardDelete(id),
    replaceUrl: (url) => replaceState(url, {}),
    navigateTo: (url) => goto(url),
  },

  refreshDrafts: () => drafts.list(),
};
