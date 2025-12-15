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
    list: async () => drafts.list(),
    create: async () => drafts.create(),
    get: async (uuid) => drafts.get(uuid),
    save: async (uuid, content) => drafts.save(uuid, content),
    archive: async (uuid) => drafts.archive(uuid),
    unarchive: async (uuid) => drafts.unarchive(uuid),
    pin: async (uuid) => drafts.pin(uuid),
    unpin: async (uuid) => drafts.unpin(uuid),
    restore: async (uuid) => drafts.restore(uuid),
    delete: async (uuid) => drafts.delete(uuid),
    hardDelete: async (uuid) => drafts.hardDelete(uuid),
  },

  refreshDrafts: () => drafts.list(),
};
