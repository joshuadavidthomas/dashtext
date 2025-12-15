import { createContext, untrack } from 'svelte';
import { useDebounce } from 'runed';
import type { DraftData, DraftAPI } from '../api/types';

/**
 * Draft - reactive draft model with derived presentation properties
 */
export class Draft {
  id: number;            // Internal ID
  uuid: string;          // Public UUID
  content = $state('');
  created_at: string;
  modified_at = $state('');
  deleted_at = $state<string | undefined>(undefined);
  archived = $state(false);
  pinned = $state(false);

  title = $derived(this.content.split('\n')[0].trim() || 'Untitled');

  previewLines = $derived.by(() => {
    const lines = this.content.split('\n').slice(1);
    return lines.filter((line) => line.trim()).slice(0, 3);
  });

  formattedModifiedAt = $derived.by(() => {
    const value = this.modified_at;
    const asNumber = parseInt(value);
    const date =
      !isNaN(asNumber) && value === String(asNumber)
        ? new Date(asNumber * 1000) // Unix timestamp (seconds)
        : new Date(value); // ISO/RFC 3339 string

    if (isNaN(date.getTime())) return 'Unknown date';

    return date.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit'
    });
  });

  isDeleted = $derived(this.deleted_at !== undefined);
  isActive = $derived(!this.deleted_at && !this.archived);

  constructor(data: DraftData & { id?: number }) {
    this.uuid = data.uuid;
    this.content = data.content;
    this.created_at = data.created_at;
    this.modified_at = data.modified_at;
    this.deleted_at = data.deleted_at;
    this.archived = data.archived || false;
    this.pinned = data.pinned || false;
    // Internal ID for backend operations (if available)
    this.id = data.id || 0;
  }
}

/**
 * DraftsState - unified state for all drafts and current draft with autosave
 * Updated to use new DraftAPI interface directly for UUID-based operations
 * Set in context by root layout, consumed by Editor, Sidebar, and route pages
 */
export class DraftsState {
  drafts = $state<Draft[]>([]);
  currentDraft = $state<Draft | null>(null);

  private pendingContent: string | null = null;
  private debouncedSave: ReturnType<typeof useDebounce>;
  private api: DraftAPI;

  constructor(initialDrafts: Draft[], api: DraftAPI) {
    this.drafts = initialDrafts;
    this.api = api;
    this.debouncedSave = useDebounce(() => this.performSave(), () => 500);
  }

  setCurrentDraft(draft: Draft | null) {
    if (draft) {
      // Use existing draft from our array to maintain object identity for reactivity
      this.currentDraft = this.drafts.find((d) => d.uuid === draft.uuid) ?? draft;
    } else {
      this.currentDraft = null;
    }
  }

  setCurrentDraftByUuid(uuid: string) {
    const draft = this.drafts.find((d) => d.uuid === uuid);
    this.currentDraft = draft || null;
  }

  updateContent(content: string) {
    if (this.currentDraft) {
      this.currentDraft.content = content;
    }
    this.pendingContent = content;
    this.debouncedSave();
  }

  async flushPendingSave(): Promise<{replaceUrl?: string, newDraftUuid?: string}> {
    // Cancel debounce and run immediately to get return value
    this.debouncedSave.cancel();
    return await this.performSave();
  }

  private async performSave(): Promise<{replaceUrl?: string, newDraftUuid?: string}> {
    if (this.pendingContent === null) return {};

    if (this.currentDraft === null && this.pendingContent.trim()) {
      // NEW DRAFT: create in DB, update local state, return navigation hint
      const newDraftData = await this.api.create();
      const newDraft = new Draft({ ...newDraftData, id: 0 }); // Internal ID not exposed
      newDraft.content = this.pendingContent;
      await this.api.save(newDraft.uuid, this.pendingContent);

      // Update local state (triggers reactive updates)
      this.drafts = [newDraft, ...this.drafts];
      this.currentDraft = newDraft;

      this.pendingContent = null;
      
      // Return navigation hint for caller
      return { replaceUrl: `/drafts/${newDraft.uuid}`, newDraftUuid: newDraft.uuid };
    } else if (this.currentDraft !== null) {
      // EXISTING DRAFT: just save
      const updated = await this.api.save(this.currentDraft.uuid, this.pendingContent);
      this.currentDraft.modified_at = updated.modified_at;
    }

    this.pendingContent = null;
    return {};
  }

  async deleteCurrentDraft(): Promise<{navigateTo?: string}> {
    if (!this.currentDraft) return {};

    const uuid = this.currentDraft.uuid;
    await this.api.delete(uuid);

    // Update local state
    this.drafts = this.drafts.filter((d) => d.uuid !== uuid);

    // Return navigation hint for caller
    if (this.drafts.length > 0) {
      return { navigateTo: `/drafts/${this.drafts[0].uuid}` };
    } else {
      return { navigateTo: '/drafts/new' };
    }
  }

  async archiveCurrentDraft(): Promise<{navigateTo?: string}> {
    if (!this.currentDraft) return {};
    
    const updated = await this.api.archive(this.currentDraft.uuid);
    this.currentDraft.archived = updated.archived || false;
    
    // Return navigation hint for caller if archived
    if (updated.archived) {
      const remainingDrafts = this.drafts.filter(d => d.uuid !== this.currentDraft!.uuid && !d.archived);
      if (remainingDrafts.length > 0) {
        return { navigateTo: `/drafts/${remainingDrafts[0].uuid}` };
      } else {
        return { navigateTo: '/drafts/new' };
      }
    }
    
    return {};
  }

  async togglePinCurrentDraft() {
    if (!this.currentDraft) return;
    
    if (this.currentDraft.pinned) {
      const updated = await this.api.unpin(this.currentDraft.uuid);
      this.currentDraft.pinned = updated.pinned || false;
    } else {
      const updated = await this.api.pin(this.currentDraft.uuid);
      this.currentDraft.pinned = updated.pinned || false;
      
      // Update pinned status on other drafts (single pin constraint)
      this.drafts.forEach(d => {
        if (d.uuid !== this.currentDraft!.uuid && d.pinned) {
          d.pinned = false;
        }
      });
    }
  }
}

export const [getDraftsState, setDraftsState] = createContext<DraftsState>();

export const createDraftsState = (getInitialDrafts: () => Draft[], api: DraftAPI) => {
  const drafts = new DraftsState(untrack(getInitialDrafts), api);
  setDraftsState(drafts);
  return drafts;
};
