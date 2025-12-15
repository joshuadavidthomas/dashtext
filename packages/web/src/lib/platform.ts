import { replaceState, goto } from '$app/navigation';
import { drafts } from '$lib/api';
import type { PlatformCapabilities } from '@dashtext/lib/platform';

export const webPlatform: PlatformCapabilities = {
  platform: 'web',

  onFocusChange(callback) {
    const handler = () => {
      if (document.visibilityState === 'visible') callback();
    };
    document.addEventListener('visibilitychange', handler);
    return () => document.removeEventListener('visibilitychange', handler);
  },

  // Not available on web
  window: null,
  quickCapture: null,
  settings: null, // Web uses dialog instead

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
