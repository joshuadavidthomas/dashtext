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
    createDraft: async () => drafts.create(),
    saveDraft: async (id, content) => drafts.save(id, content),
    deleteDraft: async (id) => drafts.delete(id),
    archiveDraft: async (id) => drafts.archive(id),
    unarchiveDraft: async (id) => drafts.unarchive(id),
    pinDraft: async (id) => drafts.pin(id),
    unpinDraft: async (id) => drafts.unpin(id),
    restoreDraft: async (id) => drafts.restore(id),
    hardDeleteDraft: async (id) => drafts.hardDelete(id),
  },

  refreshDrafts: () => drafts.list(),
};
