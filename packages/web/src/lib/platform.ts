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

  draftsAPI: {
    createDraft: async () => drafts.create(),
    saveDraft: async (id, content) => drafts.save(id, content),
    deleteDraft: async (id) => drafts.delete(id),
    replaceUrl: (url) => replaceState(url, {}),
    navigateTo: (url) => goto(url)
  },

  refreshDrafts: () => drafts.list(),
};
