import type { DraftAPI, DraftData } from './types';
import { Draft } from '$lib/stores/drafts.svelte';

export type { DraftAPI, DraftData };

let backend: DraftAPI | null = null;

/**
 * Check if running in Tauri environment
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Get the appropriate backend (lazy-loaded)
 */
async function getBackend(): Promise<DraftAPI> {
  if (backend) return backend;

  if (isTauri()) {
    const { tauriBackend } = await import('./backends/tauri');
    backend = tauriBackend;
  } else {
    // Use mock backend for browser testing (e.g., Playwright)
    const { mockBackend } = await import('./backends/mock');
    backend = mockBackend;
  }

  return backend;
}

// Convenience functions that call the appropriate backend method
// These return Draft instances for backward compatibility with the store

export async function listDrafts(): Promise<Draft[]> {
  const api = await getBackend();
  const data = await api.list();
  return data.map((d) => new Draft(d));
}

export async function createDraft(): Promise<Draft> {
  const api = await getBackend();
  const data = await api.create();
  return new Draft(data);
}

export async function getDraft(id: number): Promise<Draft | null> {
  const api = await getBackend();
  const data = await api.get(id);
  return data ? new Draft(data) : null;
}

export async function saveDraft(id: number, content: string): Promise<DraftData> {
  const api = await getBackend();
  return api.save(id, content);
}

export async function deleteDraft(id: number): Promise<void> {
  const api = await getBackend();
  return api.delete(id);
}
